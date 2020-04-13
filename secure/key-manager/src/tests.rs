// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{Action, Error, KeyManager, LibraInterface};
use executor::{db_bootstrapper, Executor};
use libra_config::config::NodeConfig;
use libra_crypto::{ed25519, HashValue, PrivateKeyExt, Uniform};
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
    on_chain_config::ConfigurationResource,
    transaction::Transaction,
    validator_config::ValidatorConfig,
    validator_info::ValidatorInfo,
    validator_set::ValidatorSet,
};
use libra_vm::LibraVM;
use libradb::LibraDB;
use rand::{rngs::StdRng, SeedableRng};
use std::{cell::RefCell, collections::BTreeMap, convert::TryFrom, sync::Arc, time::Duration};
use storage_interface::{DbReader, DbReaderWriter};

struct Node {
    account: AccountAddress,
    executor: Executor<LibraVM>,
    libra: TestLibraInterface,
    key_manager:
        KeyManager<TestLibraInterface, InMemoryStorageInternal<MockTimeService>, MockTimeService>,
    time: MockTimeService,
}

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

impl Node {
    fn setup(config: &NodeConfig) -> Self {
        let (storage, db_rw) = DbReaderWriter::wrap(LibraDB::new(&config.storage.dir()));
        db_bootstrapper::maybe_bootstrap_db::<LibraVM>(db_rw.clone(), config)
            .expect("Failed to execute genesis");
        let executor = Executor::new(db_rw);
        let libra = TestLibraInterface {
            queued_transactions: Arc::new(RefCell::new(Vec::new())),
            storage,
        };
        let time = MockTimeService::new();
        let account = config.validator_network.as_ref().unwrap().peer_id;
        let key_manager = KeyManager::new(
            account,
            "consensus_key".to_owned(),
            libra.clone(),
            setup_secure_storage(&config, time.clone()),
            time.clone(),
        );

        Self {
            account,
            executor,
            key_manager,
            libra,
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

    fn retrieve_validator_set_resource(&self) -> Result<ValidatorSet, Error> {
        let account = account_config::validator_set_address();
        let account_state = self.retrieve_account_state(account)?;
        account_state
            .get_validator_set()?
            .ok_or(Error::DataDoesNotExist("ValidatorSet"))
    }

    fn retrieve_libra_block_resource(&self) -> Result<LibraBlockResource, Error> {
        let account = account_config::association_address();
        let account_state = self.retrieve_account_state(account)?;
        account_state
            .get_libra_block_resource()?
            .ok_or(Error::DataDoesNotExist("BlockMetadata"))
    }

    pub fn retrieve_configuration_resource(&self) -> Result<ConfigurationResource, Error> {
        let account = account_config::association_address();
        let account_state = self.retrieve_account_state(account)?;
        account_state
            .get_configuration_resource()?
            .ok_or(Error::DataDoesNotExist("Configuration"))
    }

    fn take_all_transactions(&self) -> Vec<Transaction> {
        self.queued_transactions.borrow_mut().drain(0..).collect()
    }
}

impl LibraInterface for TestLibraInterface {
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

    fn last_reconfiguration(&self) -> Result<u64, Error> {
        self.retrieve_configuration_resource()
            .map(|v| v.last_reconfiguration_time())
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
            .payload()
            .iter()
            .find(|vi| vi.account_address() == &validator_account)
            .cloned()
            .ok_or(Error::ValidatorInfoNotFound(validator_account))
    }
}

#[test]
// This simple test just proves that genesis took effect and the values are in storage
fn test_ability_to_read_move_data() {
    let (config, _genesis_key) = config_builder::test_config();
    let node = Node::setup(&config);

    node.libra.last_reconfiguration().unwrap();
    node.libra.retrieve_discovery_set().unwrap();
    node.libra.retrieve_validator_config(node.account).unwrap();
    node.libra.retrieve_discovery_set().unwrap();
    node.libra.retrieve_validator_info(node.account).unwrap();
    assert!(node.libra.retrieve_libra_block_resource().is_ok());
}

#[test]
// This tests that a manual consensus key rotation can be performed by generating a new keypair,
// creating a new rotation transaction, and executing the transaction locally.
fn test_manual_consensus_rotation() {
    let (config, _genesis_key) = config_builder::test_config();
    let mut node = Node::setup(&config);

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
    let new_prikey = ed25519::PrivateKey::generate(&mut rng);
    let new_pubkey = new_prikey.public_key();

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

    assert_ne!(new_pubkey, genesis_pubkey);
    assert_eq!(new_pubkey, new_config.consensus_pubkey);
    assert_eq!(&new_pubkey, new_info.consensus_public_key());
}

#[test]
// This verifies the key manager is properly setup and that a basic rotation can occur
fn test_key_manager_init_and_basic_rotation() {
    let (config, _genesis_key) = config_builder::test_config();
    let mut node = Node::setup(&config);
    node.key_manager.compare_storage_to_config().unwrap();
    node.key_manager.compare_info_to_config().unwrap();
    assert_eq!(node.time.now(), node.key_manager.last_rotation().unwrap());
    // No reconfiguration yet
    assert_eq!(0, node.key_manager.last_reconfiguration().unwrap());
    // No executions yet
    assert_eq!(0, node.key_manager.libra_timestamp().unwrap());

    let genesis_info = node.libra.retrieve_validator_info(node.account).unwrap();
    let new_key = node.key_manager.rotate_consensus_key().unwrap();
    let pre_exe_rotated_info = node.libra.retrieve_validator_info(node.account).unwrap();
    assert_eq!(
        genesis_info.consensus_public_key(),
        pre_exe_rotated_info.consensus_public_key()
    );
    assert_ne!(pre_exe_rotated_info.consensus_public_key(), &new_key);

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
fn test_loop() {
    let (config, _genesis_key) = config_builder::test_config();
    let mut node = Node::setup(&config);

    let mut action = node.key_manager.evaluate_status().unwrap();
    assert_eq!(action, Action::NoAction);
    node.key_manager.perform_action(action).unwrap();

    node.time.increment_by(crate::ROTATION_PERIOD_SECS);
    action = node.key_manager.evaluate_status().unwrap();
    assert_eq!(action, Action::FullKeyRotation);
    node.key_manager.perform_action(action).unwrap();

    action = node.key_manager.evaluate_status().unwrap();
    assert_eq!(action, Action::NoAction);

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
