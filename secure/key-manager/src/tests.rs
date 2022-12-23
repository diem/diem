// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    diem_interface::JsonRpcDiemInterface, Action, DiemInterface, Error, KeyManager, GAS_UNIT_PRICE,
    MAX_GAS_AMOUNT,
};
use anyhow::Result;
use diem_config::{
    config::{KeyManagerConfig, NodeConfig},
    utils,
    utils::get_genesis_txn,
};
use diem_crypto::{ed25519::Ed25519PrivateKey, HashValue, PrivateKey, Uniform};
use diem_global_constants::{
    CONSENSUS_KEY, OPERATOR_ACCOUNT, OPERATOR_KEY, OWNER_ACCOUNT, OWNER_KEY,
};
use diem_secure_storage::{InMemoryStorage, KVStorage};
use diem_time_service::{MockTimeService, TimeService, TimeServiceTrait};
use diem_types::{
    account_address::AccountAddress,
    account_config,
    account_config::XDX_NAME,
    account_state::AccountState,
    block_info::BlockInfo,
    block_metadata::{BlockMetadata, DiemBlockResource},
    ledger_info::{LedgerInfo, LedgerInfoWithSignatures},
    mempool_status::{MempoolStatus, MempoolStatusCode},
    on_chain_config::{ConfigurationResource, ValidatorSet},
    transaction::{RawTransaction, Transaction},
    validator_config::ValidatorConfig,
    validator_info::ValidatorInfo,
};
use diem_vm::DiemVM;
use diemdb::DiemDB;
use executor::Executor;
use executor_types::BlockExecutor;
use futures::{channel::mpsc::channel, StreamExt};
use rand::{rngs::StdRng, SeedableRng};
use std::{cell::RefCell, collections::BTreeMap, convert::TryFrom, sync::Arc};
use storage_interface::{DbReader, DbReaderWriter, MoveDbReader};
use tokio::runtime::Runtime;
use vm_validator::{
    mocks::mock_vm_validator::MockVMValidator, vm_validator::TransactionValidation,
};

const TXN_EXPIRATION_SECS: u64 = 100;

struct Node<T: DiemInterface> {
    executor: Executor<DiemVM>,
    diem: DiemInterfaceTestHarness<T>,
    key_manager: KeyManager<DiemInterfaceTestHarness<T>, InMemoryStorage>,
    time: MockTimeService,
}

impl<T: DiemInterface> Node<T> {
    pub fn new(
        executor: Executor<DiemVM>,
        diem: DiemInterfaceTestHarness<T>,
        key_manager: KeyManager<DiemInterfaceTestHarness<T>, InMemoryStorage>,
        time: MockTimeService,
    ) -> Self {
        Self {
            executor,
            diem,
            key_manager,
            time,
        }
    }

    // Increments the diem_timestamp on the blockchain by executing an empty block.
    fn update_diem_timestamp(&mut self) {
        self.execute_and_commit(vec![]);
    }

    fn execute_and_commit(&mut self, mut block: Vec<Transaction>) {
        // 1) Update the clock for potential reconfigurations
        self.time.advance_secs(1);

        let timestamp = self.time.now_unix_time().as_micros() as u64;
        let owner_account = self.get_account_from_storage(OWNER_ACCOUNT);
        let block_id = HashValue::zero();
        let block_metadata = BlockMetadata::new(block_id, 0, timestamp, vec![], owner_account);
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
                output.epoch_state().clone(),
            ),
            HashValue::zero(),
        );
        let ledger_info_with_sigs = LedgerInfoWithSignatures::new(ledger_info, BTreeMap::new());
        self.executor
            .commit_blocks(vec![block_id], ledger_info_with_sigs)
            .unwrap();
    }

    fn get_account_from_storage(&mut self, account_name: &str) -> AccountAddress {
        self.key_manager
            .storage
            .get::<AccountAddress>(account_name)
            .unwrap()
            .value
    }

    fn get_key_from_storage(&mut self, key_name: &str) -> Ed25519PrivateKey {
        self.key_manager
            .storage
            .get::<Ed25519PrivateKey>(key_name)
            .unwrap()
            .value
    }
}

/// The struct below is a DiemInterface wrapper that exposes several additional methods to better
/// test the internal state of a DiemInterface implementation (e.g., during end-to-end and
/// integration tests).
#[derive(Clone)]
struct DiemInterfaceTestHarness<T: DiemInterface> {
    diem: T,
    submitted_transactions: Arc<RefCell<Vec<Transaction>>>,
}

impl<T: DiemInterface> DiemInterfaceTestHarness<T> {
    fn new(diem: T) -> Self {
        Self {
            diem,
            submitted_transactions: Arc::new(RefCell::new(Vec::new())),
        }
    }

    /// Returns the validator set associated with the validator set address.
    fn retrieve_validator_set(&self) -> Result<ValidatorSet, Error> {
        let account = account_config::validator_set_address();
        let account_state = self.diem.retrieve_account_state(account)?;
        account_state
            .get_validator_set()?
            .ok_or_else(|| Error::DataDoesNotExist("ValidatorSetResource".into()))
    }

    /// Returns the diem block resource associated with the association address.
    fn retrieve_diem_block_resource(&self) -> Result<DiemBlockResource, Error> {
        let account = account_config::diem_root_address();
        let account_state = self.diem.retrieve_account_state(account)?;
        account_state
            .get_diem_block_resource()?
            .ok_or_else(|| Error::DataDoesNotExist("BlockMetadata".into()))
    }

    /// Return (and clear) all transactions that have been submitted to this interface since the
    /// last time this method was called.
    fn take_all_transactions(&self) -> Vec<Transaction> {
        self.submitted_transactions.borrow_mut().drain(..).collect()
    }
}

impl<T: DiemInterface> DiemInterface for DiemInterfaceTestHarness<T> {
    fn diem_timestamp(&self) -> Result<u64, Error> {
        self.diem.diem_timestamp()
    }

    fn last_reconfiguration(&self) -> Result<u64, Error> {
        self.diem.last_reconfiguration()
    }

    fn retrieve_sequence_number(&self, account: AccountAddress) -> Result<u64, Error> {
        self.diem.retrieve_sequence_number(account)
    }

    fn submit_transaction(&self, transaction: Transaction) -> Result<(), Error> {
        self.submitted_transactions
            .borrow_mut()
            .push(transaction.clone());
        self.diem.submit_transaction(transaction)
    }

    fn retrieve_validator_config(&self, account: AccountAddress) -> Result<ValidatorConfig, Error> {
        self.diem.retrieve_validator_config(account)
    }

    fn retrieve_validator_info(&self, account: AccountAddress) -> Result<ValidatorInfo, Error> {
        self.diem.retrieve_validator_info(account)
    }

    fn retrieve_account_state(&self, account: AccountAddress) -> Result<AccountState, Error> {
        self.diem.retrieve_account_state(account)
    }
}

/// A mock diem interface implementation that stores a pointer to the DiemDB from which to
/// process API requests.
#[derive(Clone)]
struct MockDiemInterface {
    storage: Arc<DiemDB>,
}

impl MockDiemInterface {
    fn retrieve_validator_set_resource(&self) -> Result<ValidatorSet, Error> {
        let account = account_config::validator_set_address();
        let account_state = self.retrieve_account_state(account)?;
        account_state
            .get_validator_set()?
            .ok_or_else(|| Error::DataDoesNotExist("ValidatorSet".into()))
    }

    pub fn retrieve_configuration_resource(&self) -> Result<ConfigurationResource, Error> {
        let account = diem_types::on_chain_config::config_address();
        let account_state = self.retrieve_account_state(account)?;
        account_state
            .get_configuration_resource()?
            .ok_or_else(|| Error::DataDoesNotExist("Configuration".into()))
    }
}

impl DiemInterface for MockDiemInterface {
    fn diem_timestamp(&self) -> Result<u64, Error> {
        let account = account_config::diem_root_address();
        let blob = self
            .storage
            .get_latest_account_state(account)?
            .ok_or_else(|| Error::DataDoesNotExist("AccountState".into()))?;
        let account_state = AccountState::try_from(&blob)?;
        Ok(account_state
            .get_diem_timestamp_resource()?
            .ok_or_else(|| Error::DataDoesNotExist("DiemTimestampResource".into()))?
            .diem_timestamp
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
        let account_state = AccountState::try_from(&blob)?;
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
        let account_state = AccountState::try_from(&blob)?;
        account_state
            .get_validator_config_resource()?
            .ok_or_else(|| Error::DataDoesNotExist("ValidatorConfigResource".into()))?
            .validator_config
            .ok_or_else(|| {
                Error::DataDoesNotExist(format!(
                    "ValidatorConfigResource not found for account: {:?}",
                    account
                ))
            })
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

// Creates and returns NodeConfig and KeyManagerConfig structs that are consistent for testing.
fn get_test_configs() -> (NodeConfig, KeyManagerConfig) {
    let (node_config, _) = diem_genesis_tool::test_config();
    let key_manager_config = KeyManagerConfig::default();
    (node_config, key_manager_config)
}

// Creates and returns a test node that uses the JsonRpcDiemInterface.
// This setup is useful for testing nodes as they operate in a production environment.
fn setup_node_using_json_rpc() -> (Node<JsonRpcDiemInterface>, Runtime) {
    let (node_config, key_manager_config) = get_test_configs();

    let (storage, db_rw) = setup_diem_db(&node_config);
    let (diem, server) = setup_diem_interface_and_json_server(storage);
    let executor = Executor::new(db_rw);

    (
        setup_node(&node_config, &key_manager_config, executor, diem),
        server,
    )
}

// Creates and returns a Node using the MockDiemInterface implementation.
// This setup is useful for testing and verifying new development features quickly.
fn setup_node_using_test_mocks() -> Node<MockDiemInterface> {
    let (node_config, key_manager_config) = get_test_configs();
    let (storage, db_rw) = setup_diem_db(&node_config);
    let diem = MockDiemInterface { storage };
    let executor = Executor::new(db_rw);

    setup_node(&node_config, &key_manager_config, executor, diem)
}

// Creates and returns a diem database and database reader/writer pair bootstrapped with genesis.
fn setup_diem_db(config: &NodeConfig) -> (Arc<DiemDB>, DbReaderWriter) {
    let (storage, db_rw) = DbReaderWriter::wrap(DiemDB::new_for_test(&config.storage.dir()));
    executor_test_helpers::bootstrap_genesis::<DiemVM>(&db_rw, get_genesis_txn(config).unwrap())
        .expect("Failed to execute genesis");

    (storage, db_rw)
}

// Creates and returns a node for testing using the given config, executor and diem interface
// wrapper implementation.
fn setup_node<T: DiemInterface + Clone>(
    node_config: &NodeConfig,
    key_manager_config: &KeyManagerConfig,
    executor: Executor<DiemVM>,
    diem: T,
) -> Node<T> {
    let time = TimeService::mock();
    let diem_test_harness = DiemInterfaceTestHarness::new(diem);
    let storage = setup_secure_storage(node_config, time.clone());

    let key_manager = KeyManager::new(
        diem_test_harness.clone(),
        storage,
        time.clone(),
        key_manager_config.rotation_period_secs,
        key_manager_config.sleep_period_secs,
        key_manager_config.txn_expiration_secs,
        diem_types::chain_id::ChainId::test(),
    );

    Node::new(executor, diem_test_harness, key_manager, time.into_mock())
}

// Creates and returns a secure storage implementation (based on an in memory storage engine) for
// testing. As part of the initialization, the consensus key is created.
fn setup_secure_storage(config: &NodeConfig, time: TimeService) -> InMemoryStorage {
    let mut sec_storage = InMemoryStorage::new_with_time_service(time);
    let test_config = config.clone().test.unwrap();

    // Initialize the owner key and account address in storage
    let owner_key = test_config.owner_key.unwrap();
    sec_storage.set(OWNER_KEY, owner_key.private_key()).unwrap();

    let owner_account = config.consensus.safety_rules.test.as_ref().unwrap().author;
    sec_storage.set(OWNER_ACCOUNT, owner_account).unwrap();

    // Initialize the operator key and account address in storage
    let operator_key = test_config.operator_key.unwrap();
    sec_storage
        .set(OPERATOR_KEY, operator_key.private_key())
        .unwrap();

    let operator_account = diem_types::account_address::from_public_key(&operator_key.public_key());
    sec_storage.set(OPERATOR_ACCOUNT, operator_account).unwrap();

    // Initialize the consensus key in storage
    let sr_test_config = config.consensus.safety_rules.test.as_ref().unwrap();
    let consensus_prikey = sr_test_config.consensus_key.as_ref().unwrap().private_key();
    sec_storage
        .set(crate::CONSENSUS_KEY, consensus_prikey)
        .unwrap();

    sec_storage
}

// Generates and returns a (diem interface, server) pair, where the diem interface is a JSON RPC
// based interface using the lightweight JSON client internally, and the server is a JSON server
// that serves the JSON RPC requests. The server communicates with the given database reader/writer
// to handle each JSON RPC request.
fn setup_diem_interface_and_json_server(
    db_reader: Arc<dyn MoveDbReader>,
) -> (JsonRpcDiemInterface, Runtime) {
    let address = "127.0.0.1";
    let port = utils::get_available_port();
    let host = format!("{}:{}", address, port);

    let (mp_sender, mut mp_events) = channel(1024);
    let server = diem_json_rpc::test_bootstrap(host.parse().unwrap(), db_reader, mp_sender);

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

    let json_rpc_endpoint = format!("http://{}", host);
    let diem = JsonRpcDiemInterface::new(json_rpc_endpoint);

    (diem, server)
}

#[test]
// This simple test just proves that genesis took effect and the values are on-chain (in storage).
fn test_ability_to_read_move_data() {
    // Test the mock diem interface implementation
    let node = setup_node_using_test_mocks();
    verify_ability_to_read_move_data(node);

    // Test the json diem interface implementation
    let (node, _runtime) = setup_node_using_json_rpc();
    verify_ability_to_read_move_data(node);
}

fn verify_ability_to_read_move_data<T: DiemInterface>(mut node: Node<T>) {
    let owner_account = node.get_account_from_storage(OWNER_ACCOUNT);

    node.diem.last_reconfiguration().unwrap();
    node.diem.retrieve_validator_set().unwrap();
    node.diem.retrieve_validator_config(owner_account).unwrap();
    node.diem.retrieve_validator_info(owner_account).unwrap();
    node.diem.retrieve_diem_block_resource().unwrap();
}

#[test]
// This tests that a manual on-chain consensus key rotation can be performed (in the event of an
// unexpected failure). To do this, the test generates a new keypair locally, creates a new rotation
// transaction manually and executes the transaction on-chain.
fn test_manual_rotation_on_chain() {
    // Test the mock diem interface implementation
    let node = setup_node_using_test_mocks();
    verify_manual_rotation_on_chain(node);

    // Test the json diem interface implementation
    let (node, _runtime) = setup_node_using_json_rpc();
    verify_manual_rotation_on_chain(node);
}

fn verify_manual_rotation_on_chain<T: DiemInterface>(mut node: Node<T>) {
    let owner_account = node.get_account_from_storage(OWNER_ACCOUNT);
    let genesis_config = node.diem.retrieve_validator_config(owner_account).unwrap();
    let genesis_info = node.diem.retrieve_validator_info(owner_account).unwrap();

    // Check on-chain consensus state matches the genesis state
    let consensus_pubkey = node.get_key_from_storage(CONSENSUS_KEY).public_key();
    assert_eq!(consensus_pubkey, genesis_config.consensus_public_key);
    assert_eq!(&consensus_pubkey, genesis_info.consensus_public_key());
    assert_eq!(&owner_account, genesis_info.account_address());

    // Perform on-chain rotation
    let operator_privkey = node.get_key_from_storage(OPERATOR_KEY);
    let operator_account = node.get_account_from_storage(OPERATOR_ACCOUNT);
    let mut rng = StdRng::from_seed([44u8; 32]);
    let new_privkey = Ed25519PrivateKey::generate(&mut rng);
    let new_pubkey = new_privkey.public_key();
    // Increment time to 5min + 1sec, so that the rotation passes through rate limits
    node.time.advance_secs(301);
    let txn1 = crate::build_rotation_transaction(
        owner_account,
        operator_account,
        0,
        &new_pubkey,
        Vec::new(),
        Vec::new(),
        node.time.now_secs() + TXN_EXPIRATION_SECS,
        diem_types::chain_id::ChainId::test(),
    );
    let txn1 = txn1
        .sign(&operator_privkey, operator_privkey.public_key())
        .unwrap();
    let txn1 = Transaction::UserTransaction(txn1.into_inner());

    node.execute_and_commit(vec![txn1]);

    let new_config = node.diem.retrieve_validator_config(owner_account).unwrap();
    let new_info = node.diem.retrieve_validator_info(owner_account).unwrap();

    // Check on-chain consensus state has been rotated
    assert_ne!(new_pubkey, consensus_pubkey);
    assert_eq!(new_pubkey, new_config.consensus_public_key);
    assert_eq!(&new_pubkey, new_info.consensus_public_key());
}

#[test]
// This verifies that the key manager is properly setup and that a basic rotation can be performed.
fn test_key_manager_init_and_basic_rotation() {
    // Test the mock diem interface implementation
    let node = setup_node_using_test_mocks();
    verify_init_and_basic_rotation(node);

    // Test the json diem interface implementation
    let (node, _runtime) = setup_node_using_json_rpc();
    verify_init_and_basic_rotation(node);
}

fn verify_init_and_basic_rotation<T: DiemInterface>(mut node: Node<T>) {
    // Verify correct initialization (on-chain and in storage)
    node.key_manager.compare_storage_to_config().unwrap();
    node.key_manager.compare_info_to_config().unwrap();
    assert_eq!(
        node.time.now_secs(),
        node.key_manager.last_rotation().unwrap()
    );
    // No executions yet
    assert_eq!(0, node.key_manager.last_reconfiguration().unwrap());
    assert_eq!(0, node.key_manager.diem_timestamp().unwrap());

    // Perform key rotation locally
    let owner_account = node.get_account_from_storage(OWNER_ACCOUNT);
    let genesis_info = node.diem.retrieve_validator_info(owner_account).unwrap();
    let new_key = node.key_manager.rotate_consensus_key().unwrap();
    let pre_exe_rotated_info = node.diem.retrieve_validator_info(owner_account).unwrap();
    assert_eq!(
        genesis_info.consensus_public_key(),
        pre_exe_rotated_info.consensus_public_key()
    );
    assert_ne!(pre_exe_rotated_info.consensus_public_key(), &new_key);

    // Increment time to 5min + 1sec, so that the rotation passes through rate limits
    node.time.advance_secs(301);

    // Execute key rotation on-chain
    node.execute_and_commit(node.diem.take_all_transactions());
    let rotated_info = node.diem.retrieve_validator_info(owner_account).unwrap();
    assert_ne!(
        genesis_info.consensus_public_key(),
        rotated_info.consensus_public_key()
    );
    assert_eq!(rotated_info.consensus_public_key(), &new_key);

    // Executions have occurred but nothing after our rotation
    assert_ne!(0, node.key_manager.diem_timestamp().unwrap());
    assert_eq!(
        node.key_manager.last_reconfiguration(),
        node.key_manager.diem_timestamp()
    );

    // Executions have occurred after our rotation
    node.execute_and_commit(node.diem.take_all_transactions());
    assert_ne!(
        node.key_manager.last_reconfiguration(),
        node.key_manager.diem_timestamp()
    );
}

#[test]
// This tests the application's main loop to ensure it handles basic operations and reliabilities.
// To do this, the test repeatedly calls "execute_once_and_sleep" -- identical to the main "execute"
// loop.
fn test_execute() {
    // Test the mock diem interface implementation
    let node = setup_node_using_test_mocks();
    verify_execute(node);

    // Test the json diem interface implementation
    let (node, _runtime) = setup_node_using_json_rpc();
    verify_execute(node);
}

fn verify_execute<T: DiemInterface>(mut node: Node<T>) {
    let (_, key_manager_config) = get_test_configs();

    // Verify correct initial state (i.e., nothing to be done by key manager)
    assert_eq!(0, node.time.now_secs());
    assert_eq!(0, node.diem.last_reconfiguration().unwrap());

    // Verify rotation required after enough time
    node.time
        .advance_secs(key_manager_config.rotation_period_secs);
    node.update_diem_timestamp();
    assert_eq!(
        Action::FullKeyRotation,
        node.key_manager.evaluate_status().unwrap()
    );

    // Verify a single execution iteration will perform the rotation and re-sync everything
    node.update_diem_timestamp();
    node.key_manager.execute_once().unwrap();

    // Verify nothing to be done after rotation except wait for transaction execution
    node.update_diem_timestamp();
    assert_eq!(
        Action::WaitForTransactionExecution,
        node.key_manager.evaluate_status().unwrap()
    );

    // Verify rotation transaction not executed, now expired
    node.time
        .advance_secs(key_manager_config.txn_expiration_secs);
    node.update_diem_timestamp();
    assert_eq!(
        Action::SubmitKeyRotationTransaction,
        node.key_manager.evaluate_status().unwrap()
    );

    // Let's execute the expired transaction and see that a resubmission is still required
    node.execute_and_commit(node.diem.take_all_transactions());
    assert_eq!(
        Action::SubmitKeyRotationTransaction,
        node.key_manager.evaluate_status().unwrap()
    );

    // Verify that a single execution iteration will resubmit the transaction, which can then be
    // executed to re-sync everything up (on-chain).
    node.update_diem_timestamp();
    node.key_manager.execute_once().unwrap();
    node.execute_and_commit(node.diem.take_all_transactions());
    assert_eq!(
        Action::NoAction,
        node.key_manager.evaluate_status().unwrap()
    );
    assert_ne!(0, node.diem.last_reconfiguration().unwrap());
}

#[test]
// This tests the key manager's ability to detect liveness errors on the blockchain.
fn test_liveness_error() {
    // Test the mock diem interface implementation
    let node = setup_node_using_test_mocks();
    verify_liveness_error(node);

    // Test the json diem interface implementation
    let (node, _runtime) = setup_node_using_json_rpc();
    verify_liveness_error(node);
}

fn verify_liveness_error<T: DiemInterface>(mut node: Node<T>) {
    // Verify correct initial state
    assert_eq!(0, node.time.now_secs());
    assert_eq!(0, node.diem.last_reconfiguration().unwrap());

    // Verify no action is required by the key manager
    node.update_diem_timestamp();
    assert_eq!(
        Action::NoAction,
        node.key_manager.evaluate_status().unwrap()
    );

    // Verify a single execution iteration will detect a liveness error because
    // the diem timestamp on-chain hasn't been updated since it was last checked.
    match node.key_manager.execute_once() {
        Err(Error::LivenessError(last_timestamp, current_timestamp)) => {
            assert_eq!(last_timestamp, current_timestamp)
        }
        result => panic!("Expected a liveness error, but got: {:?}", result),
    }
}

#[test]
// This tests the key manager's ability to detect missing accounts in storage.
fn test_missing_account_error() {
    // Test the mock diem interface implementation
    let node = setup_node_using_test_mocks();
    verify_missing_account_error(node);

    // Test the json diem interface implementation
    let (node, _runtime) = setup_node_using_json_rpc();
    verify_missing_account_error(node);
}

fn verify_missing_account_error<T: DiemInterface>(mut node: Node<T>) {
    // Verify correct initial state
    assert_eq!(0, node.time.now_secs());
    assert_eq!(0, node.diem.last_reconfiguration().unwrap());

    // Reset storage to wipe all account addresses
    node.update_diem_timestamp();
    node.key_manager.storage.reset_and_clear().unwrap();

    // Verify a single execution iteration will detect the missing account address
    match node.key_manager.execute_once() {
        Err(Error::MissingAccountAddress(..)) => { /* Expected */ }
        result => panic!(
            "Expected a missing account address error, but got: {:?}",
            result
        ),
    }
}

#[test]
// This tests the key manager's ability to detect mismatches between the validator
// config and the validator set.
fn test_validator_config_info_mismatch() {
    // Test the mock diem interface implementation
    let node = setup_node_using_test_mocks();
    verify_validator_config_info_mismatch(node);

    // Test the json diem interface implementation
    let (node, _runtime) = setup_node_using_json_rpc();
    verify_validator_config_info_mismatch(node);
}

fn verify_validator_config_info_mismatch<T: DiemInterface>(mut node: Node<T>) {
    // Verify correct initial state
    assert_eq!(0, node.time.now_secs());
    assert_eq!(0, node.diem.last_reconfiguration().unwrap());

    // Verify no action is required by the key manager
    node.update_diem_timestamp();
    assert_eq!(
        Action::NoAction,
        node.key_manager.evaluate_status().unwrap()
    );

    // Build a transaction updating the validator config consensus key on chain
    let mut rng = StdRng::from_seed([44u8; 32]);
    let new_consensus_key = Ed25519PrivateKey::generate(&mut rng).public_key();
    let script = diem_transaction_builder::stdlib::encode_register_validator_config_script(
        node.get_account_from_storage(OWNER_ACCOUNT),
        new_consensus_key.to_bytes().to_vec(),
        Vec::new(),
        Vec::new(),
    );
    let txn = RawTransaction::new_script(
        node.get_account_from_storage(OPERATOR_ACCOUNT),
        0,
        script,
        MAX_GAS_AMOUNT,
        GAS_UNIT_PRICE,
        XDX_NAME.to_owned(),
        node.time.now_secs() + TXN_EXPIRATION_SECS,
        diem_types::chain_id::ChainId::test(),
    );

    // Sign and execute the validator config update transaction
    let operator_privkey = node.get_key_from_storage(OPERATOR_KEY);
    let txn = txn
        .sign(&operator_privkey, operator_privkey.public_key())
        .unwrap();
    let txn = Transaction::UserTransaction(txn.into_inner());
    node.execute_and_commit(vec![txn]);

    // Verify the key manager will detect a key mismatch between the validator
    // config and the validator set
    node.update_diem_timestamp();
    assert_eq!(
        Action::WaitForReconfiguration,
        node.key_manager.evaluate_status().unwrap()
    );
}

#[test]
// This tests the key manager's ability to detect mismatches between storage and
// the blockchain.
fn test_storage_blockchain_mismatch() {
    // Test the mock diem interface implementation
    let node = setup_node_using_test_mocks();
    verify_storage_blockchain_mismatch(node);

    // Test the json diem interface implementation
    let (node, _runtime) = setup_node_using_json_rpc();
    verify_storage_blockchain_mismatch(node);
}

fn verify_storage_blockchain_mismatch<T: DiemInterface>(mut node: Node<T>) {
    // Verify correct initial state
    assert_eq!(0, node.time.now_secs());
    assert_eq!(0, node.diem.last_reconfiguration().unwrap());

    // Verify no action is required by the key manager
    node.update_diem_timestamp();
    assert_eq!(
        Action::NoAction,
        node.key_manager.evaluate_status().unwrap()
    );

    // Perform key rotation locally in storage but don't execute it on-chain.
    let _ = node.key_manager.rotate_consensus_key().unwrap();

    // Verify the key manager will detect a key mismatch between storage and the
    // blockchain and thus wait for transaction execution
    node.update_diem_timestamp();
    assert_eq!(
        Action::WaitForTransactionExecution,
        node.key_manager.evaluate_status().unwrap()
    );
}

#[test]
// This tests the key manager's ability to detect generic storage errors.
fn test_storage_error() {
    // Test the mock diem interface implementation
    let node = setup_node_using_test_mocks();
    verify_storage_error(node);

    // Test the json diem interface implementation
    let (node, _runtime) = setup_node_using_json_rpc();
    verify_storage_error(node);
}

fn verify_storage_error<T: DiemInterface>(mut node: Node<T>) {
    // Verify correct initial state
    assert_eq!(0, node.time.now_secs());
    assert_eq!(0, node.diem.last_reconfiguration().unwrap());

    // Update the consensus key to an invalid value to force a storage error
    node.update_diem_timestamp();
    node.key_manager
        .storage
        .set(CONSENSUS_KEY, "invalid value")
        .unwrap();

    // Verify a single execution iteration will detect the storage error
    match node.key_manager.execute_once() {
        Err(Error::StorageError(_)) => { /* Expected */ }
        result => panic!("Expected a storage error, but got: {:?}", result),
    }
}

#[test]
// This tests the key manager's ability to handle missing data errors.
fn test_missing_data_error() {
    // Test the mock diem interface implementation
    let node = setup_node_using_test_mocks();
    verify_missing_data(node);

    // Test the json diem interface implementation
    let (node, _runtime) = setup_node_using_json_rpc();
    verify_missing_data(node);
}

fn verify_missing_data<T: DiemInterface>(mut node: Node<T>) {
    // Verify correct initial state
    assert_eq!(0, node.time.now_secs());
    assert_eq!(0, node.diem.last_reconfiguration().unwrap());

    // Update the owner account to an account that will not have any state on chain
    node.update_diem_timestamp();
    node.key_manager
        .storage
        .set(OWNER_ACCOUNT, "00000000000000000000000000000001")
        .unwrap();

    // Verify a single execution iteration will detect the missing data error
    match node.key_manager.execute_once() {
        Err(Error::DataDoesNotExist(_)) => { /* Expected */ }
        result => panic!(
            "Expected a data does not exist error, but got: {:?}",
            result
        ),
    }
}
