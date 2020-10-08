// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use executor::{
    db_bootstrapper::{generate_waypoint, maybe_bootstrap},
    Executor,
};
use executor_types::BlockExecutor;
use libra_config::{config::NodeConfig, utils::get_genesis_txn};
use libra_crypto::{
    ed25519::{Ed25519PrivateKey, Ed25519PublicKey},
    hash::HashValue,
    PrivateKey, SigningKey, Uniform,
};
use libra_logger::prelude::*;
use libra_types::{
    account_address::AccountAddress,
    account_config::{
        coin1_tmp_tag, testnet_dd_account_address, treasury_compliance_account_address,
        AccountResource, COIN1_NAME,
    },
    block_info::BlockInfo,
    chain_id::ChainId,
    ledger_info::{LedgerInfo, LedgerInfoWithSignatures},
    transaction::{
        authenticator::AuthenticationKey, RawTransaction, Script, SignedTransaction, Transaction,
    },
};
use libra_vm::LibraVM;
use libradb::LibraDB;
use rand::{rngs::StdRng, SeedableRng};
use std::{
    collections::BTreeMap,
    convert::TryFrom,
    path::PathBuf,
    sync::{mpsc, Arc},
};
use storage_client::StorageClient;
use storage_interface::{DbReader, DbReaderWriter};
use storage_service::start_storage_service_with_db;
use transaction_builder::{
    encode_create_parent_vasp_account_script, encode_peer_to_peer_with_metadata_script,
};

struct AccountData {
    private_key: Ed25519PrivateKey,
    public_key: Ed25519PublicKey,
    address: AccountAddress,
    sequence_number: u64,
}

impl AccountData {
    pub fn auth_key_prefix(&self) -> Vec<u8> {
        AuthenticationKey::ed25519(&self.public_key)
            .prefix()
            .to_vec()
    }
}

struct TransactionGenerator {
    /// The current state of the accounts. The main purpose is to keep track of the sequence number
    /// so generated transactions are guaranteed to be successfully executed.
    accounts: Vec<AccountData>,

    /// Used to mint accounts.
    genesis_key: Ed25519PrivateKey,

    /// For deterministic transaction generation.
    rng: StdRng,

    /// Each generated block of transactions are sent to this channel. Using `SyncSender` to make
    /// sure if execution is slow to consume the transactions, we do not run out of memory.
    block_sender: Option<mpsc::SyncSender<Vec<Transaction>>>,
}

impl TransactionGenerator {
    fn new(
        genesis_key: Ed25519PrivateKey,
        num_accounts: usize,
        block_sender: mpsc::SyncSender<Vec<Transaction>>,
    ) -> Self {
        let seed = [1u8; 32];
        let mut rng = StdRng::from_seed(seed);

        let mut accounts = Vec::with_capacity(num_accounts);
        for _i in 0..num_accounts {
            let private_key = Ed25519PrivateKey::generate(&mut rng);
            let public_key = private_key.public_key();
            let address = libra_types::account_address::from_public_key(&public_key);
            let account = AccountData {
                private_key,
                public_key,
                address,
                sequence_number: 0,
            };
            accounts.push(account);
        }

        Self {
            accounts,
            genesis_key,
            rng,
            block_sender: Some(block_sender),
        }
    }

    fn run(&mut self, init_account_balance: u64, block_size: usize, num_transfer_blocks: usize) {
        self.gen_account_creations(block_size);
        self.gen_mint_transactions(init_account_balance, block_size);
        self.gen_transfer_transactions(block_size, num_transfer_blocks);
    }

    fn gen_account_creations(&self, block_size: usize) {
        let tc_account = treasury_compliance_account_address();

        for (i, block) in self.accounts.chunks(block_size).enumerate() {
            let mut transactions = Vec::with_capacity(block_size);
            for (j, account) in block.iter().enumerate() {
                let txn = create_transaction(
                    tc_account,
                    (i * block_size + j) as u64,
                    &self.genesis_key,
                    self.genesis_key.public_key(),
                    encode_create_parent_vasp_account_script(
                        coin1_tmp_tag(),
                        0,
                        account.address,
                        account.auth_key_prefix(),
                        vec![],
                        false, /* add all currencies */
                    ),
                );
                transactions.push(txn);
            }

            self.block_sender
                .as_ref()
                .unwrap()
                .send(transactions)
                .unwrap();
        }
    }

    /// Generates transactions that allocate `init_account_balance` to every account.
    fn gen_mint_transactions(&self, init_account_balance: u64, block_size: usize) {
        let testnet_dd_account = testnet_dd_account_address();

        for (i, block) in self.accounts.chunks(block_size).enumerate() {
            let mut transactions = Vec::with_capacity(block_size);
            for (j, account) in block.iter().enumerate() {
                let txn = create_transaction(
                    testnet_dd_account,
                    (i * block_size + j) as u64,
                    &self.genesis_key,
                    self.genesis_key.public_key(),
                    encode_peer_to_peer_with_metadata_script(
                        coin1_tmp_tag(),
                        account.address,
                        init_account_balance,
                        vec![],
                        vec![],
                    ),
                );
                transactions.push(txn);
            }

            self.block_sender
                .as_ref()
                .unwrap()
                .send(transactions)
                .unwrap();
        }
    }

    /// Generates transactions for random pairs of accounts.
    fn gen_transfer_transactions(&mut self, block_size: usize, num_blocks: usize) {
        for _i in 0..num_blocks {
            let mut transactions = Vec::with_capacity(block_size);
            for _j in 0..block_size {
                let indices = rand::seq::index::sample(&mut self.rng, self.accounts.len(), 2);
                let sender_idx = indices.index(0);
                let receiver_idx = indices.index(1);

                let sender = &self.accounts[sender_idx];
                let receiver = &self.accounts[receiver_idx];
                let txn = create_transaction(
                    sender.address,
                    sender.sequence_number,
                    &sender.private_key,
                    sender.public_key.clone(),
                    encode_peer_to_peer_with_metadata_script(
                        coin1_tmp_tag(),
                        receiver.address,
                        1, /* amount */
                        vec![],
                        vec![],
                    ),
                );
                transactions.push(txn);

                self.accounts[sender_idx].sequence_number += 1;
            }

            self.block_sender
                .as_ref()
                .unwrap()
                .send(transactions)
                .unwrap();
        }
    }

    /// Verifies the sequence numbers in storage match what we have locally.
    fn verify_sequence_number(&self, db: &dyn DbReader) {
        for account in &self.accounts {
            let address = account.address;
            let blob = db
                .get_latest_account_state(address)
                .expect("Failed to query storage.")
                .expect("Account must exist.");
            let account_resource = AccountResource::try_from(&blob).unwrap();
            assert_eq!(account_resource.sequence_number(), account.sequence_number);
        }
    }

    /// Drops the sender to notify the receiving end of the channel.
    fn drop_sender(&mut self) {
        self.block_sender.take().unwrap();
    }
}

struct TransactionExecutor {
    executor: Executor<LibraVM>,
    parent_block_id: HashValue,
    block_receiver: mpsc::Receiver<Vec<Transaction>>,
}

impl TransactionExecutor {
    fn new(
        executor: Executor<LibraVM>,
        parent_block_id: HashValue,
        block_receiver: mpsc::Receiver<Vec<Transaction>>,
    ) -> Self {
        Self {
            executor,
            parent_block_id,
            block_receiver,
        }
    }

    fn run(&mut self) {
        let mut version = 0;

        while let Ok(transactions) = self.block_receiver.recv() {
            let num_txns = transactions.len();
            version += num_txns as u64;

            let execute_start = std::time::Instant::now();

            let block_id = HashValue::random();
            let output = self
                .executor
                .execute_block((block_id, transactions.clone()), self.parent_block_id)
                .unwrap();

            let execute_time = std::time::Instant::now().duration_since(execute_start);
            let commit_start = std::time::Instant::now();

            let block_info = BlockInfo::new(
                1,        /* epoch */
                0,        /* round, doesn't matter */
                block_id, /* id, doesn't matter */
                output.root_hash(),
                version,
                0,    /* timestamp_usecs, doesn't matter */
                None, /* next_epoch_state */
            );
            let ledger_info = LedgerInfo::new(
                block_info,
                HashValue::zero(), /* consensus_data_hash, doesn't matter */
            );
            let ledger_info_with_sigs =
                LedgerInfoWithSignatures::new(ledger_info, BTreeMap::new() /* signatures */);

            self.executor
                .commit_blocks(vec![block_id], ledger_info_with_sigs)
                .unwrap();

            self.parent_block_id = block_id;

            let commit_time = std::time::Instant::now().duration_since(commit_start);
            let total_time = execute_time + commit_time;

            info!(
                "Version: {}. execute time: {} ms. commit time: {} ms. TPS: {}.",
                version,
                execute_time.as_millis(),
                commit_time.as_millis(),
                num_txns as u128 * 1_000_000_000 / total_time.as_nanos(),
            );
        }
    }
}

fn create_storage_service_and_executor(
    config: &NodeConfig,
) -> (Arc<dyn DbReader>, Executor<LibraVM>) {
    let (db, db_rw) = DbReaderWriter::wrap(
        LibraDB::open(
            &config.storage.dir(),
            false, /* readonly */
            None,  /* pruner */
        )
        .expect("DB should open."),
    );
    let waypoint = generate_waypoint::<LibraVM>(&db_rw, get_genesis_txn(config).unwrap()).unwrap();
    maybe_bootstrap::<LibraVM>(&db_rw, get_genesis_txn(config).unwrap(), waypoint).unwrap();

    let _handle = start_storage_service_with_db(config, db.clone());
    let executor = Executor::new(
        StorageClient::new(&config.storage.address, config.storage.timeout_ms).into(),
    );

    (db, executor)
}

/// Runs the benchmark with given parameters.
pub fn run_benchmark(
    num_accounts: usize,
    init_account_balance: u64,
    block_size: usize,
    num_transfer_blocks: usize,
    db_dir: Option<PathBuf>,
) {
    let (mut config, genesis_key) = libra_genesis_tool::test_config();
    if let Some(path) = db_dir {
        config.storage.dir = path;
    }

    let (db, executor) = create_storage_service_and_executor(&config);
    let parent_block_id = executor.committed_block_id();

    let (block_sender, block_receiver) = mpsc::sync_channel(50 /* bound */);

    // Spawn two threads to run transaction generator and executor separately.
    let gen_thread = std::thread::Builder::new()
        .name("txn_generator".to_string())
        .spawn(move || {
            let mut generator = TransactionGenerator::new(genesis_key, num_accounts, block_sender);
            generator.run(init_account_balance, block_size, num_transfer_blocks);
            generator
        })
        .expect("Failed to spawn transaction generator thread.");
    let exe_thread = std::thread::Builder::new()
        .name("txn_executor".to_string())
        .spawn(move || {
            let mut exe = TransactionExecutor::new(executor, parent_block_id, block_receiver);
            exe.run();
        })
        .expect("Failed to spawn transaction executor thread.");

    // Wait for generator to finish and get back the generator.
    let mut generator = gen_thread.join().unwrap();
    // Drop the sender so the executor thread can eventually exit.
    generator.drop_sender();
    // Wait until all transactions are committed.
    exe_thread.join().unwrap();

    // Do a sanity check on the sequence number to make sure all transactions are committed.
    generator.verify_sequence_number(db.as_ref());
}

fn create_transaction(
    sender: AccountAddress,
    sequence_number: u64,
    private_key: &Ed25519PrivateKey,
    public_key: Ed25519PublicKey,
    program: Script,
) -> Transaction {
    let now = libra_infallible::duration_since_epoch();
    let expiration_time = now.as_secs() + 3600;

    let raw_txn = RawTransaction::new_script(
        sender,
        sequence_number,
        program,
        1_000_000,             /* max_gas_amount */
        0,                     /* gas_unit_price */
        COIN1_NAME.to_owned(), /* gas_currency_code */
        expiration_time,
        ChainId::test(),
    );

    let signature = private_key.sign(&raw_txn);
    let signed_txn = SignedTransaction::new(raw_txn, public_key, signature);
    Transaction::UserTransaction(signed_txn)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_benchmark() {
        super::run_benchmark(
            25,   /* num_accounts */
            10,   /* init_account_balance */
            5,    /* block_size */
            5,    /* num_transfer_blocks */
            None, /* db_dir */
        );
    }
}
