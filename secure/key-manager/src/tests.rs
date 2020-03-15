// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use executor::Executor;
use libra_config::config::NodeConfig;
use libra_crypto::ed25519::Ed25519PublicKey;
use libra_types::{
    account_address::AccountAddress, account_config, account_state::AccountState,
    discovery_set::DiscoverySet, validator_config::ValidatorConfig, validator_set::ValidatorSet,
};
use libra_vm::LibraVM;
use libradb::{LibraDB, LibraDBTrait};
use std::{convert::TryFrom, sync::Arc};
use storage_client::{StorageReadServiceClient, StorageWriteServiceClient};
use tokio::runtime::Runtime;

struct Node {
    _executor: Arc<Executor<LibraVM>>,
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
        _executor: executor,
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
    let account_state: AccountState = AccountState::try_from(&blob).unwrap();
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

#[test]
// This simple test just proves that genesis took effect and the values are in storage
fn test() {
    let (config, _genesis_key) = config_builder::test_config();
    let node = setup(&config);

    let validator_address = config.validator_network.unwrap().peer_id;

    get_discovery_set(&node.storage);
    get_validator_config(&node.storage, validator_address);
    get_validator_set(&node.storage);
}
