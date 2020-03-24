// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{Error, KeyManager, LibraInterface};
use executor::Executor;
use libra_config::config::NodeConfig;
use libra_crypto::{ed25519::Ed25519PrivateKey, HashValue, PrivateKey, Uniform};
use libra_secure_storage::{InMemoryStorage, KVStorage, Policy, Value};
use libra_types::{
    account_address::AccountAddress,
    account_config,
    account_state::AccountState,
    block_info::BlockInfo,
    block_metadata::{BlockMetadata, LibraBlockResource},
    discovery_set::DiscoverySet,
    ledger_info::{LedgerInfo, LedgerInfoWithSignatures},
    transaction::Transaction,
    validator_config::ValidatorConfig,
    validator_info::ValidatorInfo,
    validator_set::ValidatorSetResource,
};
use libra_vm::LibraVM;
use libradb::{LibraDB, LibraDBTrait};
use rand::{rngs::StdRng, SeedableRng};
use std::{cell::RefCell, collections::BTreeMap, convert::TryFrom, sync::Arc, time::SystemTime};
use storage_client::{StorageReadServiceClient, StorageWriteServiceClient};
use tokio::runtime::Runtime;

struct Node {
    account: AccountAddress,
    executor: Executor<LibraVM>,
    libra: TestLibraInterface,
    key_manager: KeyManager<TestLibraInterface, InMemoryStorage>,
    storage: Arc<LibraDB>,
    _storage_service: Runtime,
}

fn setup_secure_storage(config: &NodeConfig) -> InMemoryStorage {
    let mut sec_storage = InMemoryStorage::new();
    let test_config = config.clone().test.unwrap();

    let mut a_keypair = test_config.account_keypair.unwrap();
    let a_prikey = Value::Ed25519PrivateKey(a_keypair.take_private().unwrap());

    sec_storage
        .create(crate::ACCOUNT_KEY, a_prikey, &Policy::public())
        .unwrap();

    let mut c_keypair = test_config.consensus_keypair.unwrap();
    let c_prikey = c_keypair.take_private().unwrap();
    let c_prikey0 = Value::Ed25519PrivateKey(c_prikey.clone());
    let c_prikey1 = Value::Ed25519PrivateKey(c_prikey);

    sec_storage
        .create(crate::CONSENSUS_KEY, c_prikey0, &Policy::public())
        .unwrap();
    // Ugly hack but we need this until we support retrieving a policy from within storage and that
    // currently is not easy, since we would need to convert from Vault -> Libra policy.
    sec_storage
        .create(
            &format!("{}_previous", crate::CONSENSUS_KEY),
            c_prikey1,
            &Policy::public(),
        )
        .unwrap();
    sec_storage
}

fn setup(config: &NodeConfig) -> Node {
    let storage = storage_service::init_libra_db(config);
    let storage_service = storage_service::start_storage_service_with_db(&config, storage.clone());
    let executor = Executor::new(
        Arc::new(StorageReadServiceClient::new(&config.storage.address)),
        Arc::new(StorageWriteServiceClient::new(&config.storage.address)),
        config,
    );
    let libra = TestLibraInterface {
        queued_transactions: Arc::new(RefCell::new(Vec::new())),
        storage: storage.clone(),
    };
    let account = config.validator_network.as_ref().unwrap().peer_id;
    let key_manager = KeyManager::new(
        account,
        "consensus_key".to_owned(),
        libra.clone(),
        setup_secure_storage(&config),
    );

    Node {
        account,
        executor,
        key_manager,
        libra,
        storage,
        _storage_service: storage_service,
    }
}

#[derive(Clone)]
struct TestLibraInterface {
    queued_transactions: Arc<RefCell<Vec<Transaction>>>,
    storage: Arc<LibraDB>,
}

impl TestLibraInterface {
    fn retrieve_account_state(&self, account: AccountAddress) -> Result<AccountState, Error> {
        let blob = self
            .storage
            .get_latest_account_state(account)?
            .ok_or(Error::DataDoesNotExist("AccountState"))?;
        Ok(AccountState::try_from(&blob)?)
    }

    fn retrieve_discovery_set(&self) -> Result<DiscoverySet, Error> {
        let account = account_config::discovery_set_address();
        let account_state = self.retrieve_account_state(account)?;
        Ok(account_state
            .get_discovery_set_resource()?
            .ok_or(Error::DataDoesNotExist("DiscoverySetResource"))?
            .discovery_set()
            .clone())
    }

    fn retrieve_validator_set_resource(&self) -> Result<ValidatorSetResource, Error> {
        let account = account_config::validator_set_address();
        let account_state = self.retrieve_account_state(account)?;
        account_state
            .get_validator_set_resource()?
            .ok_or(Error::DataDoesNotExist("ValidatorSetResource"))
    }

    fn retrieve_libra_block_resource(&self) -> Result<LibraBlockResource, Error> {
        let account = account_config::association_address();
        let account_state = self.retrieve_account_state(account)?;
        account_state
            .get_libra_block_resource()?
            .ok_or(Error::DataDoesNotExist("BlockMetadata"))
    }

    fn take_all_transactions(&self) -> Vec<Transaction> {
        self.queued_transactions.borrow_mut().drain(0..).collect()
    }
}

impl LibraInterface for TestLibraInterface {
    fn last_reconfiguration(&self) -> Result<u64, Error> {
        self.retrieve_validator_set_resource()
            .map(|v| v.last_reconfiguration_time())
    }

    fn libra_timestamp(&self) -> Result<u64, Error> {
        let account = account_config::association_address();
        let blob = self
            .storage
            .get_latest_account_state(account)?
            .ok_or(Error::DataDoesNotExist("AccountState"))?;
        let account_state = AccountState::try_from(&blob).unwrap();
        Ok(account_state
            .get_libra_timestamp_resource()?
            .ok_or(Error::DataDoesNotExist("LibraTimestampResource"))?
            .libra_timestamp
            .microseconds)
    }

    fn retrieve_sequence_number(&self, account: AccountAddress) -> Result<u64, Error> {
        let blob = self
            .storage
            .get_latest_account_state(account)?
            .ok_or(Error::DataDoesNotExist("AccountState"))?;
        let account_state = AccountState::try_from(&blob).unwrap();
        Ok(account_state
            .get_account_resource()?
            .ok_or(Error::DataDoesNotExist("AccountResource"))?
            .sequence_number())
    }

    fn submit_transaction(&self, transaction: Transaction) -> Result<(), Error> {
        self.queued_transactions.borrow_mut().push(transaction);
        Ok(())
    }

    fn retrieve_validator_config(&self, account: AccountAddress) -> Result<ValidatorConfig, Error> {
        let blob = self
            .storage
            .get_latest_account_state(account)?
            .ok_or(Error::DataDoesNotExist("AccountState"))?;
        let account_state = AccountState::try_from(&blob).unwrap();
        Ok(account_state
            .get_validator_config_resource()?
            .ok_or(Error::DataDoesNotExist("ValidatorConfigResource"))?
            .validator_config)
    }

    fn retrieve_validator_info(
        &self,
        validator_account: AccountAddress,
    ) -> Result<ValidatorInfo, Error> {
        self.retrieve_validator_set_resource()?
            .validator_set()
            .iter()
            .find(|vi| vi.account_address() == &validator_account)
            .cloned()
            .ok_or(Error::ValidatorInfoNotFound(validator_account))
    }
}

// TODO(davidiw): Notify external upon reconfigure and update epoch via ledger_info and also add
// next validator set if appropriate...
fn execute_and_commit(node: &mut Node, mut block: Vec<Transaction>) {
    let block_id = HashValue::zero();
    let startup_info = node.storage.get_startup_info().unwrap().unwrap();
    // Clock is supposed to be in microseconds
    let clock = (startup_info.committed_tree_state.version + 1) * 1_000_000;

    // This will set the current clock to be equal to the current version + 1, this guarantees that
    // the clock is always refreshed for this set of transactions.
    let block_metadata = BlockMetadata::new(block_id, 0, clock, vec![], node.account);
    let prologue = Transaction::BlockMetadata(block_metadata);
    block.insert(0, prologue);

    let result = node
        .executor
        .execute_block((block_id, block), node.executor.committed_block_id())
        .unwrap();

    let ledger_info = LedgerInfo::new(
        BlockInfo::new(
            0,
            0,
            block_id,
            result.root_hash(),
            result.version(),
            0,
            None,
        ),
        HashValue::zero(),
    );
    let ledger_info_with_sigs = LedgerInfoWithSignatures::new(ledger_info, BTreeMap::new());

    node.executor
        .commit_blocks(vec![block_id], ledger_info_with_sigs)
        .unwrap();
}

#[test]
// This simple test just proves that genesis took effect and the values are in storage
fn test_ability_to_read_move_data() {
    let (config, _genesis_key) = config_builder::test_config();
    let node = setup(&config);

    node.libra.last_reconfiguration().unwrap();
    node.libra.retrieve_discovery_set().unwrap();
    node.libra.retrieve_validator_config(node.account).unwrap();
    node.libra.retrieve_discovery_set().unwrap();
    node.libra.retrieve_validator_info(node.account).unwrap();
    assert!(node.libra.retrieve_libra_block_resource().is_ok());
}

#[test]
// This test verifies that a node can rotate its key and it results in a new validator set
fn test_consensus_rotation() {
    let (config, _genesis_key) = config_builder::test_config();
    let mut node = setup(&config);

    let test_config = config.test.unwrap();
    let mut keypair = test_config.consensus_keypair.unwrap();

    let genesis_prikey = keypair.take_private().unwrap();
    let genesis_pubkey = genesis_prikey.public_key();
    let account_prikey = test_config.account_keypair.unwrap().take_private().unwrap();

    let genesis_config = node.libra.retrieve_validator_config(node.account).unwrap();
    let genesis_info = node.libra.retrieve_validator_info(node.account).unwrap();

    assert_eq!(genesis_pubkey, genesis_config.consensus_pubkey);
    assert_eq!(&genesis_pubkey, genesis_info.consensus_public_key());
    assert_eq!(&node.account, genesis_info.account_address());

    let mut rng = StdRng::from_seed([44u8; 32]);
    let new_prikey = Ed25519PrivateKey::generate_for_testing(&mut rng);
    let new_pubkey = new_prikey.public_key();

    let txn = crate::build_transaction(node.account, 0, &account_prikey, &new_pubkey);

    execute_and_commit(&mut node, vec![txn]);

    let new_config = node.libra.retrieve_validator_config(node.account).unwrap();
    let new_info = node.libra.retrieve_validator_info(node.account).unwrap();

    assert!(new_pubkey != genesis_pubkey);
    assert_eq!(new_pubkey, new_config.consensus_pubkey);
    assert_eq!(&new_pubkey, new_info.consensus_public_key());
}

#[test]
// This verifies the key manager is properly setup and that a
fn test_key_manager_init_and_basic_rotation() {
    let (config, _genesis_key) = config_builder::test_config();
    let mut node = setup(&config);
    node.key_manager.compare_storage_to_config().unwrap();
    node.key_manager.compare_info_to_config().unwrap();
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    assert!(now - 600 <= node.key_manager.last_rotation().unwrap());
    // No reconfiguration yet
    assert_eq!(0, node.key_manager.last_reconfiguration().unwrap());
    // No executions yet
    assert_eq!(0, node.key_manager.libra_timestamp().unwrap());

    let genesis_info = node.libra.retrieve_validator_info(node.account).unwrap();
    let new_key = node.key_manager.rotate_consensus_key().unwrap();
    let pre_exe_rotated_info = node.libra.retrieve_validator_info(node.account).unwrap();
    assert!(genesis_info.consensus_public_key() == pre_exe_rotated_info.consensus_public_key());
    assert!(pre_exe_rotated_info.consensus_public_key() != &new_key);

    let txns = node.libra.take_all_transactions();
    execute_and_commit(&mut node, txns);
    let rotated_info = node.libra.retrieve_validator_info(node.account).unwrap();
    assert!(genesis_info.consensus_public_key() != rotated_info.consensus_public_key());
    assert!(rotated_info.consensus_public_key() == &new_key);

    // Executions have occurred but nothing after our rotation
    assert!(0 != node.key_manager.libra_timestamp().unwrap());
    assert_eq!(
        node.key_manager.last_reconfiguration(),
        node.key_manager.libra_timestamp()
    );

    // TODO(davidiw): Enable once reconfiguration is correclty implemented above...
    // Executions have occurred after our rotation
    // execute_and_commit(&node, node.libra.take_all_transactions());
    // assert!(node.key_manager.last_reconfiguration() != node.key_manager.libra_timestamp());
}
