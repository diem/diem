// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{libra_interface::JsonRpcLibraInterface, Action, Error, KeyManager, LibraInterface};
use anyhow::Result;
use executor::{db_bootstrapper, BlockExecutor, Executor};
use futures::{channel::mpsc::channel, StreamExt};
use libra_config::{config::NodeConfig, utils, utils::get_genesis_txn};
use libra_crypto::{ed25519::Ed25519PrivateKey, HashValue, PrivateKey, Uniform};
use libra_secure_json_rpc::JsonRpcClient;
use libra_secure_storage::{InMemoryStorageInternal, KVStorage, Policy, Value};
use libra_secure_time::{MockTimeService, TimeService};
use libra_types::{
    account_address::AccountAddress,
    account_config,
    account_state::AccountState,
    block_info::BlockInfo,
    block_metadata::{BlockMetadata, LibraBlockResource},
    discovery_set::DiscoverySet,
    ledger_info::{LedgerInfo, LedgerInfoWithSignatures},
    mempool_status::{MempoolStatus, MempoolStatusCode},
    on_chain_config::{ConfigurationResource, ValidatorSet},
    transaction::Transaction,
    validator_config::ValidatorConfig,
    validator_info::ValidatorInfo,
};
use libra_vm::LibraVM;
use libradb::LibraDB;
use rand::{rngs::StdRng, SeedableRng};
use std::{cell::RefCell, collections::BTreeMap, convert::TryFrom, sync::Arc, time::Duration};
use storage_interface::{DbReader, DbReaderWriter};
use tokio::runtime::Runtime;
use vm_validator::{
    mocks::mock_vm_validator::MockVMValidator, vm_validator::TransactionValidation,
};

struct Node<T: LibraInterface> {
    account: AccountAddress,
    executor: Executor<LibraVM>,
    libra: LibraInterfaceTestHarness<T>,
    key_manager: KeyManager<
        LibraInterfaceTestHarness<T>,
        InMemoryStorageInternal<MockTimeService>,
        MockTimeService,
    >,
    time: MockTimeService,
}

impl<T: LibraInterface> Node<T> {
    pub fn new(
        account: AccountAddress,
        executor: Executor<LibraVM>,
        libra: LibraInterfaceTestHarness<T>,
        key_manager: KeyManager<
            LibraInterfaceTestHarness<T>,
            InMemoryStorageInternal<MockTimeService>,
            MockTimeService,
        >,
        time: MockTimeService,
    ) -> Self {
        Self {
            account,
            executor,
            libra,
            key_manager,
            time,
        }
    }

    fn execute_and_commit(&mut self, mut block: Vec<Transaction>) {
        // 1) Update the clock for potential reconfigurations
        self.time.increment();
        // Clock is supposed to be in microseconds
        let clock = self.time.now() * 1_000_000;

        let block_id = HashValue::zero();
        let block_metadata = BlockMetadata::new(block_id, 0, clock, vec![], self.account);
        let prologue = Transaction::BlockMetadata(block_metadata);
        block.insert(0, prologue);

        // 2) Execute the transactions
        let output = self
            .executor
            .execute_block((block_id, block), self.executor.committed_block_id())
            .unwrap();

        // 3) Produce a new LI and commit the executed blocks
        let ledger_info = LedgerInfo::new(
            BlockInfo::new(
                1,
                0,
                block_id,
                output.root_hash(),
                output.version(),
                0,
                output.validators().clone(),
            ),
            HashValue::zero(),
        );
        let ledger_info_with_sigs = LedgerInfoWithSignatures::new(ledger_info, BTreeMap::new());
        self.executor
            .commit_blocks(vec![block_id], ledger_info_with_sigs)
            .unwrap();
    }
}

/// The struct below is a LibraInterface wrapper that exposes several additional methods to better
/// test the internal state of a LibraInterface implementation (e.g., during end-to-end and
/// integration tests).
#[derive(Clone)]
struct LibraInterfaceTestHarness<T: LibraInterface> {
    libra: T,
    submitted_transactions: Arc<RefCell<Vec<Transaction>>>,
}

impl<T: LibraInterface> LibraInterfaceTestHarness<T> {
    fn new(libra: T) -> Self {
        Self {
            libra,
            submitted_transactions: Arc::new(RefCell::new(Vec::new())),
        }
    }

    /// Returns the discover set associated with the discovery set address.
    fn retrieve_discovery_set(&self) -> Result<DiscoverySet, Error> {
        let account = account_config::discovery_set_address();
        let account_state = self.libra.retrieve_account_state(account)?;
        Ok(account_state
            .get_discovery_set_resource()?
            .ok_or_else(|| Error::DataDoesNotExist("DiscoverySetResource".into()))?
            .discovery_set()
            .clone())
    }

    /// Returns the libra block resource associated with the association address.
    fn retrieve_libra_block_resource(&self) -> Result<LibraBlockResource, Error> {
        let account = account_config::association_address();
        let account_state = self.libra.retrieve_account_state(account)?;
        account_state
            .get_libra_block_resource()?
            .ok_or_else(|| Error::DataDoesNotExist("BlockMetadata".into()))
    }

    /// Return (and clear) all transactions that have been submitted to this interface since the
    /// last time this method was called.
    fn take_all_transactions(&self) -> Vec<Transaction> {
        self.submitted_transactions.borrow_mut().drain(..).collect()
    }
}

impl<T: LibraInterface> LibraInterface for LibraInterfaceTestHarness<T> {
    fn libra_timestamp(&self) -> Result<u64, Error> {
        self.libra.libra_timestamp()
    }

    fn last_reconfiguration(&self) -> Result<u64, Error> {
        self.libra.last_reconfiguration()
    }

    fn retrieve_sequence_number(&self, account: AccountAddress) -> Result<u64, Error> {
        self.libra.retrieve_sequence_number(account)
    }

    fn submit_transaction(&self, transaction: Transaction) -> Result<(), Error> {
        self.submitted_transactions
            .borrow_mut()
            .push(transaction.clone());
        self.libra.submit_transaction(transaction)
    }

    fn retrieve_validator_config(&self, account: AccountAddress) -> Result<ValidatorConfig, Error> {
        self.libra.retrieve_validator_config(account)
    }

    fn retrieve_validator_info(&self, account: AccountAddress) -> Result<ValidatorInfo, Error> {
        self.libra.retrieve_validator_info(account)
    }

    fn retrieve_account_state(&self, account: AccountAddress) -> Result<AccountState, Error> {
        self.libra.retrieve_account_state(account)
    }
}

/// A mock libra interface implementation that stores a pointer to the LibraDB from which to
/// process API requests.
#[derive(Clone)]
struct MockLibraInterface {
    storage: Arc<LibraDB>,
}

impl MockLibraInterface {
    fn retrieve_validator_set_resource(&self) -> Result<ValidatorSet, Error> {
        let account = account_config::validator_set_address();
        let account_state = self.retrieve_account_state(account)?;
        account_state
            .get_validator_set()?
            .ok_or_else(|| Error::DataDoesNotExist("ValidatorSet".into()))
    }

    pub fn retrieve_configuration_resource(&self) -> Result<ConfigurationResource, Error> {
        let account = account_config::association_address();
        let account_state = self.retrieve_account_state(account)?;
        account_state
            .get_configuration_resource()?
            .ok_or_else(|| Error::DataDoesNotExist("Configuration".into()))
    }
}

impl LibraInterface for MockLibraInterface {
    fn libra_timestamp(&self) -> Result<u64, Error> {
        let account = account_config::association_address();
        let blob = self
            .storage
            .get_latest_account_state(account)?
            .ok_or_else(|| Error::DataDoesNotExist("AccountState".into()))?;
        let account_state = AccountState::try_from(&blob).unwrap();
        Ok(account_state
            .get_libra_timestamp_resource()?
            .ok_or_else(|| Error::DataDoesNotExist("LibraTimestampResource".into()))?
            .libra_timestamp
            .microseconds)
    }

    fn last_reconfiguration(&self) -> Result<u64, Error> {
        self.retrieve_configuration_resource()
            .map(|v| v.last_reconfiguration_time())
    }

    fn retrieve_sequence_number(&self, account: AccountAddress) -> Result<u64, Error> {
        let blob = self
            .storage
            .get_latest_account_state(account)?
            .ok_or_else(|| Error::DataDoesNotExist("AccountState".into()))?;
        let account_state = AccountState::try_from(&blob).unwrap();
        Ok(account_state
            .get_account_resource()?
            .ok_or_else(|| Error::DataDoesNotExist("AccountResource".into()))?
            .sequence_number())
    }

    fn submit_transaction(&self, _transaction: Transaction) -> Result<(), Error> {
        Ok(())
    }

    fn retrieve_validator_config(&self, account: AccountAddress) -> Result<ValidatorConfig, Error> {
        let blob = self
            .storage
            .get_latest_account_state(account)?
            .ok_or_else(|| Error::DataDoesNotExist("AccountState".into()))?;
        let account_state = AccountState::try_from(&blob).unwrap();
        Ok(account_state
            .get_validator_config_resource()?
            .ok_or_else(|| Error::DataDoesNotExist("ValidatorConfigResource".into()))?
            .validator_config)
    }

    fn retrieve_validator_info(
        &self,
        validator_account: AccountAddress,
    ) -> Result<ValidatorInfo, Error> {
        self.retrieve_validator_set_resource()?
            .payload()
            .iter()
            .find(|vi| vi.account_address() == &validator_account)
            .cloned()
            .ok_or(Error::ValidatorInfoNotFound(validator_account))
    }

    fn retrieve_account_state(&self, account: AccountAddress) -> Result<AccountState, Error> {
        let blob = self
            .storage
            .get_latest_account_state(account)?
            .ok_or_else(|| Error::DataDoesNotExist("AccountState".into()))?;
        Ok(AccountState::try_from(&blob)?)
    }
}

// Creates and returns a test node that uses the JsonRpcLibraInterface.
// This setup is useful for testing nodes as they operate in a production environment.
fn setup_node_using_json_rpc(config: &NodeConfig) -> (Node<JsonRpcLibraInterface>, Runtime) {
    let (_storage, db_rw) = setup_libra_db(config);
    let (client, server) = setup_json_client_and_server(db_rw.clone());
    let libra = JsonRpcLibraInterface::new(client);
    let executor = Executor::new(db_rw);

    (setup_node(config, executor, libra), server)
}

// Creates and returns a Node using the MockLibraInterface implementation.
// This setup is useful for testing and verifying new development features quickly.
fn setup_node_using_test_mocks(config: &NodeConfig) -> Node<MockLibraInterface> {
    let (storage, db_rw) = setup_libra_db(config);
    let libra = MockLibraInterface { storage };
    let executor = Executor::new(db_rw);

    setup_node(config, executor, libra)
}

// Creates and returns a libra database and database reader/writer pair bootstrapped with genesis.
fn setup_libra_db(config: &NodeConfig) -> (Arc<LibraDB>, DbReaderWriter) {
    let (storage, db_rw) = DbReaderWriter::wrap(LibraDB::new(&config.storage.dir()));
    db_bootstrapper::bootstrap_db_if_empty::<LibraVM>(&db_rw, get_genesis_txn(config).unwrap())
        .expect("Failed to execute genesis");

    (storage, db_rw)
}

// Creates and returns a node for testing using the given config, executor and libra interface
// wrapper implementation.
fn setup_node<T: LibraInterface + Clone>(
    config: &NodeConfig,
    executor: Executor<LibraVM>,
    libra: T,
) -> Node<T> {
    let account = config.validator_network.as_ref().unwrap().peer_id;
    let time = MockTimeService::new();
    let libra_test_harness = LibraInterfaceTestHarness::new(libra);
    let key_manager = KeyManager::new(
        account,
        crate::CONSENSUS_KEY.into(),
        libra_test_harness.clone(),
        setup_secure_storage(&config, time.clone()),
        time.clone(),
    );

    Node::new(account, executor, libra_test_harness, key_manager, time)
}

// Creates and returns a secure storage implementation (based on an in memory storage engine) for
// testing. As part of the initialization, the consensus key is created.
fn setup_secure_storage(
    config: &NodeConfig,
    time: MockTimeService,
) -> InMemoryStorageInternal<MockTimeService> {
    let mut sec_storage = InMemoryStorageInternal::new_with_time_service(time);
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

// Generates and returns a (client, server) pair, where the client is a lightweight JSON client
// and the server is a JSON server that serves the JSON RPC requests. The server communicates
// with the given database reader/writer to handle each JSON RPC request.
fn setup_json_client_and_server(db_rw: DbReaderWriter) -> (JsonRpcClient, Runtime) {
    let address = "0.0.0.0";
    let port = utils::get_available_port();
    let host = format!("{}:{}", address, port);

    let (mp_sender, mut mp_events) = channel(1024);
    let server = libra_json_rpc::bootstrap(host.parse().unwrap(), db_rw.reader, mp_sender);

    // Provide a VMValidator to the runtime.
    server.spawn(async move {
        while let Some((txn, cb)) = mp_events.next().await {
            let vm_status = MockVMValidator.validate_transaction(txn).unwrap().status();
            let result = if vm_status.is_some() {
                (MempoolStatus::new(MempoolStatusCode::VmError), vm_status)
            } else {
                (MempoolStatus::new(MempoolStatusCode::Accepted), None)
            };
            cb.send(Ok(result)).unwrap();
        }
    });

    let url = format!("http://{}", host);
    let client = JsonRpcClient::new(url);

    (client, server)
}

#[test]
// This simple test just proves that genesis took effect and the values are in storage
fn test_ability_to_read_move_data() {
    // Test the mock libra interface implementation
    let (config, _genesis_key) = config_builder::test_config();
    let node = setup_node_using_test_mocks(&config);
    verify_ability_to_read_move_data(node);

    // Test the json libra interface implementation
    let (config, _genesis_key) = config_builder::test_config();
    let (node, _runtime) = setup_node_using_json_rpc(&config);
    verify_ability_to_read_move_data(node);
}

fn verify_ability_to_read_move_data<T: LibraInterface>(node: Node<T>) {
    assert!(node.libra.last_reconfiguration().is_ok());
    assert!(node.libra.retrieve_discovery_set().is_ok());
    assert!(node.libra.retrieve_validator_config(node.account).is_ok());
    assert!(node.libra.retrieve_discovery_set().is_ok());
    assert!(node.libra.retrieve_validator_info(node.account).is_ok());
    assert!(node.libra.retrieve_libra_block_resource().is_ok());
}

#[test]
// This tests that a manual consensus key rotation can be performed by generating a new keypair,
// creating a new rotation transaction, and executing the transaction locally.
fn test_manual_consensus_rotation() {
    // Test the mock libra interface implementation
    let (config, _genesis_key) = config_builder::test_config();
    let node = setup_node_using_test_mocks(&config);
    verify_manual_consensus_rotation(config, node);

    // Test the json libra interface implementation
    let (config, _genesis_key) = config_builder::test_config();
    let (node, _runtime) = setup_node_using_json_rpc(&config);
    verify_manual_consensus_rotation(config, node);
}

fn verify_manual_consensus_rotation<T: LibraInterface>(config: NodeConfig, mut node: Node<T>) {
    let test_config = config.test.unwrap();
    let account_prikey = test_config.account_keypair.unwrap().take_private().unwrap();
    let genesis_pubkey = test_config
        .consensus_keypair
        .unwrap()
        .take_private()
        .unwrap()
        .public_key();

    let genesis_config = node.libra.retrieve_validator_config(node.account).unwrap();
    let genesis_info = node.libra.retrieve_validator_info(node.account).unwrap();

    // Check on-chain consensus state matches the genesis state
    assert_eq!(genesis_pubkey, genesis_config.consensus_pubkey);
    assert_eq!(&genesis_pubkey, genesis_info.consensus_public_key());
    assert_eq!(&node.account, genesis_info.account_address());

    // Perform rotation
    let mut rng = StdRng::from_seed([44u8; 32]);
    let new_pubkey = Ed25519PrivateKey::generate(&mut rng).public_key();
    let txn = crate::build_rotation_transaction(
        node.account,
        0,
        &account_prikey,
        &new_pubkey,
        Duration::from_secs(node.time.now() + 100),
    );
    node.execute_and_commit(vec![txn]);

    let new_config = node.libra.retrieve_validator_config(node.account).unwrap();
    let new_info = node.libra.retrieve_validator_info(node.account).unwrap();

    // Check on-chain consensus state has been rotated
    assert_ne!(new_pubkey, genesis_pubkey);
    assert_eq!(new_pubkey, new_config.consensus_pubkey);
    assert_eq!(&new_pubkey, new_info.consensus_public_key());
}

#[test]
// This verifies the key manager is properly setup and that a basic rotation can occur
fn test_key_manager_init_and_basic_rotation() {
    // Test the mock libra interface implementation
    let (config, _genesis_key) = config_builder::test_config();
    let node = setup_node_using_test_mocks(&config);
    verify_init_and_basic_rotation(node);

    // Test the json libra interface implementation
    let (config, _genesis_key) = config_builder::test_config();
    let (node, _runtime) = setup_node_using_json_rpc(&config);
    verify_init_and_basic_rotation(node);
}

fn verify_init_and_basic_rotation<T: LibraInterface>(mut node: Node<T>) {
    // Verify correct initialization (on-chain and in storage)
    assert!(node.key_manager.compare_storage_to_config().is_ok());
    assert!(node.key_manager.compare_info_to_config().is_ok());
    assert_eq!(node.time.now(), node.key_manager.last_rotation().unwrap());
    // No executions yet
    assert_eq!(0, node.key_manager.last_reconfiguration().unwrap());
    assert_eq!(0, node.key_manager.libra_timestamp().unwrap());

    // Perform key rotation locally
    let genesis_info = node.libra.retrieve_validator_info(node.account).unwrap();
    let new_key = node.key_manager.rotate_consensus_key().unwrap();
    let pre_exe_rotated_info = node.libra.retrieve_validator_info(node.account).unwrap();
    assert_eq!(
        genesis_info.consensus_public_key(),
        pre_exe_rotated_info.consensus_public_key()
    );
    assert_ne!(pre_exe_rotated_info.consensus_public_key(), &new_key);

    // Execute key rotation on-chain
    node.execute_and_commit(node.libra.take_all_transactions());
    let rotated_info = node.libra.retrieve_validator_info(node.account).unwrap();
    assert_ne!(
        genesis_info.consensus_public_key(),
        rotated_info.consensus_public_key()
    );
    assert_eq!(rotated_info.consensus_public_key(), &new_key);

    // Executions have occurred but nothing after our rotation
    assert_ne!(0, node.key_manager.libra_timestamp().unwrap());
    assert_eq!(
        node.key_manager.last_reconfiguration(),
        node.key_manager.libra_timestamp()
    );

    // Executions have occurred after our rotation
    node.execute_and_commit(node.libra.take_all_transactions());
    assert_ne!(
        node.key_manager.last_reconfiguration(),
        node.key_manager.libra_timestamp()
    );
}

#[test]
// This tests the application's main loop to ensure it handles basic operations and reliabilities
fn test_main_loop() {
    // Test the mock libra interface implementation
    let (config, _genesis_key) = config_builder::test_config();
    let node = setup_node_using_test_mocks(&config);
    verify_main_loop(node);

    // Test the json libra interface implementation
    let (config, _genesis_key) = config_builder::test_config();
    let (node, _runtime) = setup_node_using_json_rpc(&config);
    verify_main_loop(node);
}

fn verify_main_loop<T: LibraInterface>(mut node: Node<T>) {
    // Verify nothing to be done initially
    let mut action = node.key_manager.evaluate_status().unwrap();
    assert_eq!(action, Action::NoAction);
    node.key_manager.perform_action(action).unwrap();

    // Verify rotation required after enough time elapsed
    node.time.increment_by(crate::ROTATION_PERIOD_SECS);
    action = node.key_manager.evaluate_status().unwrap();
    assert_eq!(action, Action::FullKeyRotation);
    node.key_manager.perform_action(action).unwrap();

    // Verify rotation was performed, nothing to be done
    action = node.key_manager.evaluate_status().unwrap();
    assert_eq!(action, Action::NoAction);

    // Verify transaction not executed, now expired
    node.time.increment_by(crate::TXN_RETRY_SECS);
    action = node.key_manager.evaluate_status().unwrap();
    assert_eq!(action, Action::SubmitKeyRotationTransaction);

    // Let's execute the expired transaction! And then a good transaction!
    node.execute_and_commit(node.libra.take_all_transactions());
    action = node.key_manager.evaluate_status().unwrap();
    assert_eq!(action, Action::SubmitKeyRotationTransaction);
    node.key_manager.perform_action(action).unwrap();
    node.execute_and_commit(node.libra.take_all_transactions());

    action = node.key_manager.evaluate_status().unwrap();
    assert_eq!(action, Action::NoAction);
}
