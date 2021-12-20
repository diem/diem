// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::driver_factory::DriverFactory;
use consensus_notifications::ConsensusNotifier;
use data_streaming_service::streaming_client::new_streaming_service_client_listener_pair;
use diem_config::config::{NodeConfig, RoleType};
use diem_crypto::{
    ed25519::{Ed25519PrivateKey, Ed25519Signature},
    HashValue, PrivateKey, Uniform,
};
use diem_data_client::diemnet::DiemNetDataClient;
use diem_infallible::RwLock;
use diem_time_service::TimeService;
use diem_types::{
    account_address::AccountAddress,
    block_info::BlockInfo,
    chain_id::ChainId,
    ledger_info::{LedgerInfo, LedgerInfoWithSignatures},
    move_resource::MoveStorage,
    on_chain_config::ON_CHAIN_CONFIG_REGISTRY,
    transaction::{
        RawTransaction, Script, SignedTransaction, Transaction, TransactionPayload, Version,
        WriteSetPayload,
    },
    waypoint::Waypoint,
};
use diem_vm::DiemVM;
use diemdb::DiemDB;
use event_notifications::{
    EventNotificationSender, EventSubscriptionService, ReconfigNotificationListener,
};
use executor::chunk_executor::ChunkExecutor;
use executor_test_helpers::bootstrap_genesis;
use mempool_notifications::MempoolNotificationListener;
use network::application::{interface::MultiNetworkSender, storage::PeerMetadataStorage};
use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc,
};
use storage_interface::{DbReader, DbReaderWriter};
use storage_service_client::StorageServiceClient;

/// Creates a state sync driver with the given config and waypoint
#[allow(dead_code)]
pub fn create_driver_with_config_and_waypoint(
    node_config: NodeConfig,
    waypoint: Waypoint,
) -> (
    DriverFactory,
    ConsensusNotifier,
    MempoolNotificationListener,
    ReconfigNotificationListener,
) {
    create_driver_for_tests(node_config, waypoint)
}

/// Creates a state sync driver for a validator node
pub fn create_validator_driver() -> (
    DriverFactory,
    ConsensusNotifier,
    MempoolNotificationListener,
    ReconfigNotificationListener,
) {
    let mut node_config = NodeConfig::default();
    node_config.base.role = RoleType::Validator;

    create_driver_for_tests(node_config, Waypoint::default())
}

/// Creates a state sync driver for a full node
pub fn create_full_node_driver() -> (
    DriverFactory,
    ConsensusNotifier,
    MempoolNotificationListener,
    ReconfigNotificationListener,
) {
    let mut node_config = NodeConfig::default();
    node_config.base.role = RoleType::FullNode;

    create_driver_for_tests(node_config, Waypoint::default())
}

/// Creates a state sync driver using the given node config and waypoint
fn create_driver_for_tests(
    node_config: NodeConfig,
    waypoint: Waypoint,
) -> (
    DriverFactory,
    ConsensusNotifier,
    MempoolNotificationListener,
    ReconfigNotificationListener,
) {
    // Create test diem database
    let db_path = diem_temppath::TempPath::new();
    db_path.create_as_dir().unwrap();
    let (db, db_rw) = DbReaderWriter::wrap(DiemDB::new_for_test(db_path.path()));

    // Bootstrap the genesis transaction
    let (genesis, _) = vm_genesis::test_genesis_change_set_and_validators(Some(1));
    let genesis_txn = Transaction::GenesisTransaction(WriteSetPayload::Direct(genesis));
    bootstrap_genesis::<DiemVM>(&db_rw, &genesis_txn).unwrap();

    // Create the event subscription service and notify initial configs
    let storage: Arc<dyn DbReader> = db;
    let synced_version = (&*storage).fetch_synced_version().unwrap();
    let mut event_subscription_service = EventSubscriptionService::new(
        ON_CHAIN_CONFIG_REGISTRY,
        Arc::new(RwLock::new(db_rw.clone())),
    );
    let reconfiguration_subscriber = event_subscription_service
        .subscribe_to_reconfigurations()
        .unwrap();
    event_subscription_service
        .notify_initial_configs(synced_version)
        .unwrap();

    // Create consensus and mempool notifiers and listeners
    let (consensus_notifier, consensus_listener) =
        consensus_notifications::new_consensus_notifier_listener_pair(1000);
    let (mempool_notifier, mempool_listener) =
        mempool_notifications::new_mempool_notifier_listener_pair();

    // Create the chunk executor
    let chunk_executor = Arc::new(ChunkExecutor::<DiemVM>::new(db_rw.clone()).unwrap());

    // Create a streaming service client
    let (streaming_service_client, _) = new_streaming_service_client_listener_pair();

    // Create a test diem data client
    let network_client = StorageServiceClient::new(
        MultiNetworkSender::new(HashMap::new()),
        PeerMetadataStorage::new(&[]),
    );
    let (diem_data_client, _) = DiemNetDataClient::new(
        node_config.state_sync.diem_data_client,
        node_config.state_sync.storage_service,
        TimeService::mock(),
        network_client,
    );

    // Create and spawn the driver
    let driver_factory = DriverFactory::create_and_spawn_driver(
        false,
        &node_config,
        waypoint,
        db_rw,
        chunk_executor,
        mempool_notifier,
        consensus_listener,
        event_subscription_service,
        diem_data_client,
        streaming_service_client,
    );

    (
        driver_factory,
        consensus_notifier,
        mempool_listener,
        reconfiguration_subscriber,
    )
}

/// Creates a single test transaction
pub fn create_test_transaction() -> Transaction {
    let private_key = Ed25519PrivateKey::generate_for_testing();
    let public_key = private_key.public_key();

    let transaction_payload = TransactionPayload::Script(Script::new(vec![], vec![], vec![]));
    let raw_transaction = RawTransaction::new(
        AccountAddress::random(),
        0,
        transaction_payload,
        0,
        0,
        "".into(),
        0,
        ChainId::new(10),
    );
    let signed_transaction = SignedTransaction::new(
        raw_transaction,
        public_key,
        Ed25519Signature::dummy_signature(),
    );

    Transaction::UserTransaction(signed_transaction)
}

/// Creates a new ledger info with signatures at the specified version
pub fn create_ledger_info_at_version(version: Version) -> LedgerInfoWithSignatures {
    let block_info = BlockInfo::new(0, 0, HashValue::zero(), HashValue::zero(), version, 0, None);
    let ledger_info = LedgerInfo::new(block_info, HashValue::random());
    LedgerInfoWithSignatures::new(ledger_info, BTreeMap::new())
}
