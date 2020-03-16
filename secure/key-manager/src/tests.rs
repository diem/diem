// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use executor::Executor;
use executor_types::ExecutedTrees;
use libra_config::config::NodeConfig;
use libra_crypto::{
    ed25519::{Ed25519PrivateKey, Ed25519PublicKey},
    HashValue, PrivateKey, Uniform,
};
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
    validator_set::ValidatorSet,
};
use libra_vm::LibraVM;
use libradb::{LibraDB, LibraDBTrait};
use rand::{rngs::StdRng, SeedableRng};
use std::{collections::BTreeMap, convert::TryFrom, sync::Arc};
use storage_client::{StorageReadServiceClient, StorageWriteServiceClient};
use tokio::runtime::Runtime;

struct Node {
    account: AccountAddress,
    executor: Arc<Executor<LibraVM>>,
    storage: Arc<LibraDB>,
    _storage_service: Runtime,
}

fn setup(config: &NodeConfig) -> Node {
    let storage = storage_service::init_libra_db(config);
    let storage_service = storage_service::start_storage_service_with_db(&config, storage.clone());
    let executor = Arc::new(Executor::new(
        Arc::new(StorageReadServiceClient::new(&config.storage.address)),
        Arc::new(StorageWriteServiceClient::new(&config.storage.address)),
        config,
    ));

    Node {
        account: config.validator_network.as_ref().unwrap().peer_id,
        executor,
        storage,
        _storage_service: storage_service,
    }
}

fn get_discovery_set(storage: &Arc<LibraDB>) -> DiscoverySet {
    let account = account_config::discovery_set_address();
    let blob = storage.get_latest_account_state(account).unwrap().unwrap();
    let account_state = AccountState::try_from(&blob).unwrap();
    account_state
        .get_discovery_set_resource()
        .unwrap()
        .unwrap()
        .discovery_set()
        .clone()
}

fn get_validator_config(storage: &Arc<LibraDB>, account: AccountAddress) -> ValidatorConfig {
    let blob = storage.get_latest_account_state(account).unwrap().unwrap();
    let account_state = AccountState::try_from(&blob).unwrap();
    account_state
        .get_validator_config_resource()
        .unwrap()
        .unwrap()
        .validator_config
}

fn get_validator_set(storage: &Arc<LibraDB>) -> ValidatorSet<Ed25519PublicKey> {
    let account = account_config::validator_set_address();
    let blob = storage.get_latest_account_state(account).unwrap().unwrap();
    let account_state = AccountState::try_from(&blob).unwrap();
    account_state
        .get_validator_set_resource()
        .unwrap()
        .unwrap()
        .validator_set()
        .clone()
}

fn execute_and_commit(node: &Node, mut block: Vec<Transaction>) {
    let block_id = HashValue::zero();
    let startup_info = node.storage.get_startup_info().unwrap().unwrap();
    let clock = startup_info.committed_tree_state.version + 1;
    let committed_trees = ExecutedTrees::from(startup_info.committed_tree_state);

    // This will set the current clock to be equal to the current version + 1, this guarantees that
    // the clock is always refreshed for this set of transactions.
    let block_metadata = BlockMetadata::new(block_id, clock, BTreeMap::new(), node.account);
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
fn test() {
    let (config, _genesis_key) = config_builder::test_config();
    let node = setup(&config);

    get_discovery_set(&node.storage);
    get_validator_config(&node.storage, node.account);
    get_validator_set(&node.storage);
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

    let genesis_config = get_validator_config(&node.storage, node.account);
    let genesis_set = get_validator_set(&node.storage);

    assert_eq!(genesis_pubkey, genesis_config.consensus_pubkey);
    assert_eq!(&genesis_pubkey, genesis_set[0].consensus_public_key());
    assert_eq!(&node.account, genesis_set[0].account_address());

    let mut rng = StdRng::from_seed([44u8; 32]);
    let new_prikey = Ed25519PrivateKey::generate_for_testing(&mut rng);
    let new_pubkey = new_prikey.public_key();

    let txn = crate::build_transaction(node.account, 0, &account_prikey, &new_pubkey);

    execute_and_commit(&node, vec![txn]);

    let new_config = get_validator_config(&node.storage, node.account);
    let new_set = get_validator_set(&node.storage);

    assert!(new_pubkey != genesis_pubkey);
    assert_eq!(new_pubkey, new_config.consensus_pubkey);
    assert_eq!(&new_pubkey, new_set[0].consensus_public_key());
}
