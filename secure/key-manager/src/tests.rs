// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{Error, KeyManager, LibraInterface};
use executor::Executor;
use executor_types::ExecutedTrees;
use libra_config::config::NodeConfig;
use libra_crypto::{ed25519::Ed25519PrivateKey, HashValue, PrivateKey, Uniform};
use libra_secure_storage::{InMemoryStorage, KVStorage, Policy, Value};
use libra_types::{
    account_address::AccountAddress,
    account_config,
    account_state::AccountState,
    block_info::BlockInfo,
    block_metadata::BlockMetadata,
    discovery_set::DiscoverySet,
    ledger_info::{LedgerInfo, LedgerInfoWithSignatures},
    transaction::Transaction,
    validator_config::ValidatorConfig,
    validator_info::ValidatorInfo,
};
use libra_vm::LibraVM;
use libradb::{LibraDB, LibraDBTrait};
use rand::{rngs::StdRng, SeedableRng};
use std::{collections::BTreeMap, convert::TryFrom, sync::Arc, time::SystemTime};
use storage_client::{StorageReadServiceClient, StorageWriteServiceClient};
use tokio::runtime::Runtime;

struct Node {
    account: AccountAddress,
    executor: Arc<Executor<LibraVM>>,
    libra: TestLibraInterface,
    key_manager: KeyManager<TestLibraInterface, InMemoryStorage>,
    storage: Arc<LibraDB>,
    _storage_service: Runtime,
}

fn setup_secure_storage(config: &NodeConfig) -> InMemoryStorage {
    let test_config = config.clone().test.unwrap();
    let mut keypair = test_config.consensus_keypair.unwrap();
    let prikey = Value::Ed25519PrivateKey(keypair.take_private().unwrap());

    let mut sec_storage = InMemoryStorage::new();
    sec_storage
        .create(crate::CONSENSUS_KEY, prikey, &Policy::public())
        .unwrap();
    sec_storage
}

fn setup(config: &NodeConfig) -> Node {
    let storage = storage_service::init_libra_db(config);
    let storage_service = storage_service::start_storage_service_with_db(&config, storage.clone());
    let executor = Arc::new(Executor::new(
        Arc::new(StorageReadServiceClient::new(&config.storage.address)),
        Arc::new(StorageWriteServiceClient::new(&config.storage.address)),
        config,
    ));
    let libra = TestLibraInterface {
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
    storage: Arc<LibraDB>,
}

impl TestLibraInterface {
    fn retrieve_discovery_set(&self) -> Result<DiscoverySet, Error> {
        let account = account_config::discovery_set_address();
        let blob = self
            .storage
            .get_latest_account_state(account)?
            .ok_or(Error::DataDoesNotExist("AccountState"))?;
        let account_state = AccountState::try_from(&blob).unwrap();
        Ok(account_state
            .get_discovery_set_resource()?
            .ok_or(Error::DataDoesNotExist("DiscoverySetResource"))?
            .discovery_set()
            .clone())
    }
}

impl LibraInterface for TestLibraInterface {
    fn last_reconfiguration(&self) -> Result<u64, Error> {
        let account = account_config::association_address();
        let blob = self
            .storage
            .get_latest_account_state(account)?
            .ok_or(Error::DataDoesNotExist("AccountState"))?;
        let account_state = AccountState::try_from(&blob).unwrap();
        Ok(account_state
            .get_libra_timestamp_resource()?
            .ok_or(Error::DataDoesNotExist("ValidatorConfigResource"))?
            .libra_timestamp
            .microseconds)
    }

    fn retrieve_sequence_id(&self, _account: AccountAddress) -> Result<u64, Error> {
        Ok(0)
    }

    fn submit_transaction(&self, _transaction: Transaction) -> Result<(), Error> {
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
        let set_account = account_config::validator_set_address();
        let blob = self
            .storage
            .get_latest_account_state(set_account)?
            .ok_or(Error::DataDoesNotExist("AccountState"))?;
        let account_state = AccountState::try_from(&blob)?;
        let validator_set_resource = account_state
            .get_validator_set_resource()?
            .ok_or(Error::DataDoesNotExist("ValidatorSetResource"))?;
        validator_set_resource
            .validator_set()
            .iter()
            .find(|vi| vi.account_address() == &validator_account)
            .cloned()
            .ok_or(Error::ValidatorInfoNotFound(validator_account))
    }
}

fn execute_and_commit(node: &Node, mut block: Vec<Transaction>) {
    let block_id = HashValue::zero();
    let startup_info = node.storage.get_startup_info().unwrap().unwrap();
    let clock = startup_info.committed_tree_state.version + 1;
    let committed_trees = ExecutedTrees::from(startup_info.committed_tree_state);

    // This will set the current clock to be equal to the current version + 1, this guarantees that
    // the clock is always refreshed for this set of transactions.
    let block_metadata = BlockMetadata::new(block_id, 0, clock, BTreeMap::new(), node.account);
    let prologue = Transaction::BlockMetadata(block_metadata);
    block.insert(0, prologue);

    let output = node
        .executor
        .execute_block(block_id, block.clone(), &committed_trees, &committed_trees)
        .unwrap();

    let root_hash = output.accu_root();
    let version = output.version().unwrap();
    let ledger_info = LedgerInfo::new(
        BlockInfo::new(0, 0, block_id, root_hash, version, 0, None),
        HashValue::zero(),
    );
    let ledger_info_with_sigs = LedgerInfoWithSignatures::new(ledger_info, BTreeMap::new());
    let executed_block = vec![(block, Arc::new(output))];

    node.executor
        .commit_blocks(executed_block, ledger_info_with_sigs, &committed_trees)
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
}

#[test]
// This test verifies that a node can rotate its key and it results in a new validator set
fn test_consensus_rotation() {
    let (config, _genesis_key) = config_builder::test_config();
    let node = setup(&config);

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

    execute_and_commit(&node, vec![txn]);

    let new_config = node.libra.retrieve_validator_config(node.account).unwrap();
    let new_info = node.libra.retrieve_validator_info(node.account).unwrap();

    assert!(new_pubkey != genesis_pubkey);
    assert_eq!(new_pubkey, new_config.consensus_pubkey);
    assert_eq!(&new_pubkey, new_info.consensus_public_key());
}

#[test]
// This verifies the key manager is properly setup
fn test_key_manager_init() {
    let (config, _genesis_key) = config_builder::test_config();
    let node = setup(&config);
    node.key_manager.compare_storage_to_config().unwrap();
    node.key_manager.compare_info_to_config().unwrap();
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    assert!(now - 600 <= node.key_manager.last_rotation().unwrap());
    // No reconfiguration yet
    assert_eq!(0, node.key_manager.last_reconfiguration().unwrap());
}
