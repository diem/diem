use failure::prelude::*;
use std::{sync::Arc, pin::Pin};
use crate::{
    LibraDB,
    change_set::{ChangeSet, SealedChangeSet},
    errors::LibraDbError,
    event_store::EventStore,
    ledger_store::LedgerStore,
    pruner::Pruner,
    schema::*,
    state_store::StateStore,
    system_store::SystemStore,
    transaction_store::TransactionStore,
};
use types::{account_state_blob::{AccountStateBlob, AccountStateWithProof},
            transaction::{TransactionToCommit, TransactionListWithProof, Version, SignedTransactionWithProof,
                          TransactionInfo}, get_with_proof::{ResponseItem, RequestItem},
            proof::{SparseMerkleProof, AccountStateProof, EventProof, SignedTransactionProof},
            account_address::AccountAddress, access_path::AccessPath,
            contract_event::EventWithProof, account_config::AccountResource,
            ledger_info::LedgerInfoWithSignatures};
use itertools::{zip_eq, izip};
use schemadb::{DB, ColumnFamilyOptionsMap, DEFAULT_CF_NAME, ColumnFamilyOptions};
use std::time::Instant;
use logger::prelude::*;
use crypto::{ed25519::*, hash::CryptoHash, HashValue};
use jellyfish_merkle::{
    node_type::{Node, NodeKey},
    JellyfishMerkleTree, TreeReader,
};
use std::ops::Deref;
use std::sync::{Mutex, mpsc};

pub struct StartupInfo {
    pub latest_version: u64,
    pub account_state_root_hash: HashValue,
    pub transaction_accumulator_hash: HashValue,
}

pub trait ReadData: Send + Sync {
    fn update_to_latest_ledger(
        &self,
        client_known_version: Version,
        request_items: Vec<RequestItem>,
    ) -> Result<Vec<ResponseItem>>;

    fn get_transactions(
        &self,
        start_version: Version,
        batch_size: u64,
        ledger_version: Version,
        fetch_events: bool,
    ) -> Result<TransactionListWithProof>;

    fn get_account_state_with_proof_by_version(
        &self,
        address: AccountAddress,
        version: Version,
    ) -> Result<(Option<AccountStateBlob>, SparseMerkleProof)>;

    fn get_startup_info(&self) -> Result<Option<StartupInfo>>;

    fn get_state_node(&self, node_key: &NodeKey) -> Result<Option<Node>>;

    fn genesis_state(&self) -> Result<Option<bool>>;

    fn latest_version(&self) -> Option<Version>;

    fn latest_state_root(&self) -> Option<HashValue>;
}

pub trait WriteData {
    fn save_transactions(
        &self,
        txns_to_commit: Vec<TransactionToCommit>,
        first_version: Version,
    ) -> Result<()>;

    fn save_genesis_transactions(
        &self,
        txns_to_commit: Vec<TransactionToCommit>,
        ledger_info_with_sigs: &Option<LedgerInfoWithSignatures<Ed25519Signature>>,
    ) -> Result<()>;
}

pub struct DataStorage {
    libra_db: Arc<Option<LibraDB>>,
    read_db: ReadDataStorage,
    shutdown_w_sender: Mutex<mpsc::Sender<()>>,
//    shutdown_r_receiver_vec: Arc<Vec<mpsc::Receiver<()>>>,
}

impl Deref for DataStorage {
    type Target = LibraDB;

    fn deref(&self) -> &Self::Target {
        self.libra_db.as_ref().as_ref().expect("LibraDB is dropped unexptectedly")
    }
}

impl Drop for DataStorage {
    fn drop(&mut self) {
        info!("{}", "shutdown write db.");
    }
}

#[derive(Clone)]
pub struct ReadDataStorage {
    libra_db: Arc<Option<LibraDB>>,
    shutdown_r_sender: Arc<Mutex<mpsc::Sender<()>>>,
}

impl Deref for ReadDataStorage {
    type Target = LibraDB;

    fn deref(&self) -> &Self::Target {
        &self.libra_db.as_ref().as_ref().expect("LibraDB is dropped unexptectedly")
    }
}

impl Drop for ReadDataStorage {
    fn drop(&mut self) {
        info!("{}", "shutdown read db.");
    }
}

impl DataStorage {
    pub fn new(db: LibraDB) -> (Self, mpsc::Receiver<()>) {
        let write_db = Arc::new(Some(db));
        let (shutdown_w_sender, shutdown_w_receiver) = mpsc::channel();
        let (shutdown_r_sender, shutdown_r_receiver) = mpsc::channel();
        let receivers = Arc::new(vec![shutdown_r_receiver]);
        (DataStorage {
            libra_db: Arc::clone(&write_db),
            read_db: ReadDataStorage { libra_db: Arc::clone(&write_db), shutdown_r_sender: Arc::new(Mutex::new(shutdown_r_sender)) },
            shutdown_w_sender: Mutex::new(shutdown_w_sender),
            //shutdown_r_receiver_vec: receivers,
        },shutdown_w_receiver)
    }

    pub fn genesis_state(&self) -> Result<Option<bool>> {
        already_init(self.deref())
    }

    pub fn read_db(&self) -> ReadDataStorage {
        self.read_db.clone()
    }
}

fn already_init(libra_db: &LibraDB) -> Result<Option<bool>> {
    match libra_db.get_startup_info()? {
        Some(info) => Ok(Some(true)),
        None => Ok(Some(false))
    }
}

impl WriteData for DataStorage {
    fn save_transactions(
        &self,
        txns_to_commit: Vec<TransactionToCommit>,
        first_version: Version,
    ) -> Result<()> {
        let init_flag = self.genesis_state()?;
        match init_flag {
            Some(flag) => {
                ensure!(flag, "Storage has not been initialized yet.");
                self.deref().save_transactions(txns_to_commit.as_slice(), first_version, &None)
            }
            None => {
                Err(format_err!("save tx err."))
            }
        }
    }

    fn save_genesis_transactions(
        &self,
        txns_to_commit: Vec<TransactionToCommit>,
        ledger_info_with_sigs: &Option<LedgerInfoWithSignatures<Ed25519Signature>>,
    ) -> Result<()> {
        let init_flag = self.genesis_state()?;
        match init_flag {
            Some(flag) => {
                ensure!(!flag, "Storage has been initialized.");
                self.deref().save_transactions(txns_to_commit.as_slice(), 0, ledger_info_with_sigs)
            }
            None => {
                Err(format_err!("save genesis tx err."))
            }
        }
    }
}

impl ReadDataStorage {
    fn get_latest_version(&self) -> Result<Option<Version>> {
        match self.get_startup_info()? {
            Some(info) => { Ok(Some(info.latest_version)) }
            None => Ok(None),
        }
    }

    fn get_latest_state_root(&self) -> Result<Option<HashValue>> {
        match self.get_startup_info()? {
            Some(info) => { Ok(Some(info.account_state_root_hash)) }
            None => Ok(None),
        }
    }
}

impl ReadData for ReadDataStorage {
    fn update_to_latest_ledger(
        &self,
        client_known_version: Version,
        request_items: Vec<RequestItem>,
    ) -> Result<Vec<ResponseItem>> {
        let resp = self.deref().update_to_latest_ledger(client_known_version, request_items)?;
        Ok(resp.0)
    }

    fn get_transactions(
        &self,
        start_version: Version,
        batch_size: u64,
        ledger_version: Version,
        fetch_events: bool,
    ) -> Result<TransactionListWithProof> {
        self.deref().get_transactions(start_version, batch_size, ledger_version, fetch_events)
    }

    fn get_account_state_with_proof_by_version(
        &self,
        address: AccountAddress,
        version: Version,
    ) -> Result<(Option<AccountStateBlob>, SparseMerkleProof)> {
        self.deref().get_account_state_with_proof_by_version(address, version)
    }

    fn get_startup_info(&self) -> Result<Option<StartupInfo>> {
        let resp_info = self.deref().get_startup_info()?;
        match resp_info {
            Some(info) => {
                let latest_version = info.latest_version;
                let account_state_root_hash = info.account_state_root_hash;
                let transaction_accumulator_hash = info.ledger_info.transaction_accumulator_hash();
                Ok(Some(StartupInfo { latest_version, account_state_root_hash, transaction_accumulator_hash }))
            }
            None => { Ok(None) }
        }
    }

    fn get_state_node(&self, node_key: &NodeKey) -> Result<Option<Node>> {
        self.deref().state_store.get_node_option(node_key)
    }

    fn genesis_state(&self) -> Result<Option<bool>> {
        already_init(self.deref())
    }

    fn latest_version(&self) -> Option<Version> {
        self.get_latest_version().expect("get latest version err.")
    }

    fn latest_state_root(&self) -> Option<HashValue> {
        self.get_latest_state_root().expect("get latest state root err.")
    }
}

