// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use consensus_notifications::ConsensusNotificationListener;
use data_streaming_service::{
    streaming_client::new_streaming_service_client_listener_pair,
    streaming_service::DataStreamingService,
};
use diem_config::{config::NodeConfig, network_id::NetworkId};
use diem_data_client::diemnet::DiemNetDataClient;
use diem_types::{move_resource::MoveStorage, waypoint::Waypoint};
use event_notifications::{EventNotificationSender, EventSubscriptionService};
use executor_types::ChunkExecutor;
use futures::executor::block_on;
use mempool_notifications::MempoolNotificationSender;
use network::protocols::network::AppConfig;
use state_sync_driver::driver_factory::DriverFactory;
use state_sync_v1::{
    bootstrapper::StateSyncBootstrapper,
    network::{StateSyncEvents, StateSyncSender},
};
use storage_interface::default_protocol::DbReaderWriter;
use tokio::runtime::Builder;

/// A multiplexer allowing multiple versions of state sync to operate
/// concurrently (i.e., state sync v1 and state sync v2).
pub struct StateSyncMultiplexer {
    state_sync_v1: Option<StateSyncBootstrapper>,
    _state_sync_v2: Option<DriverFactory>,
}

impl StateSyncMultiplexer {
    pub fn new<M: MempoolNotificationSender + 'static>(
        network: Vec<(NetworkId, StateSyncSender, StateSyncEvents)>,
        mempool_notifier: M,
        consensus_listener: ConsensusNotificationListener,
        storage: DbReaderWriter,
        executor: Box<dyn ChunkExecutor>,
        node_config: &NodeConfig,
        waypoint: Waypoint,
        mut event_subscription_service: EventSubscriptionService,
        diem_data_client: DiemNetDataClient,
    ) -> Self {
        // Notify subscribers of the initial on-chain config values
        match (&*storage.reader).fetch_synced_version() {
            Ok(synced_version) => {
                if let Err(error) =
                    event_subscription_service.notify_initial_configs(synced_version)
                {
                    panic!(
                        "Failed to notify subscribers of initial on-chain configs: {:?}",
                        error
                    )
                }
            }
            Err(error) => panic!("Failed to fetch the initial synced version: {:?}", error),
        }

        // TODO(joshlind): update this to support also running v1 in read-only mode
        // Start state sync (with the version depending on the config)
        let mut state_sync_v1 = None;
        let mut state_sync_v2 = None;
        if node_config
            .state_sync
            .state_sync_driver
            .activate_state_sync_v2
        {
            // Create the data streaming service
            let (streaming_service_client, streaming_service_listener) =
                new_streaming_service_client_listener_pair();
            let data_streaming_service = DataStreamingService::new(
                node_config.state_sync.data_streaming_service,
                diem_data_client,
                streaming_service_listener,
            );

            // Create a new runtime for the data streaming service
            let streaming_service_runtime = Builder::new_multi_thread()
                .thread_name("data-streaming-service")
                .enable_all()
                .build()
                .expect("Failed to create data streaming service!");
            streaming_service_runtime.spawn(data_streaming_service.start_service());

            // Start state sync v2
            state_sync_v2 = Some(DriverFactory::create_and_spawn_driver(
                true,
                node_config,
                waypoint,
                storage,
                executor,
                mempool_notifier,
                consensus_listener,
                event_subscription_service,
                streaming_service_client,
            ));
        } else {
            state_sync_v1 = Some(StateSyncBootstrapper::bootstrap(
                network,
                mempool_notifier,
                consensus_listener,
                storage.reader,
                executor,
                node_config,
                waypoint,
                event_subscription_service,
                false,
            ));
        }

        Self {
            state_sync_v1,
            _state_sync_v2: state_sync_v2,
        }
    }

    pub fn block_until_initialized(&self) {
        let state_sync_v1_client = self
            .state_sync_v1
            .as_ref()
            .expect("State sync v1 is not running!")
            .create_client();
        block_on(state_sync_v1_client.wait_until_initialized())
            .expect("State sync v1 initialization failure");
    }
}

/// Configuration for the network endpoints to support state sync.
pub fn state_sync_v1_network_config() -> AppConfig {
    state_sync_v1::network::network_endpoint_config()
}

#[cfg(any(test, feature = "fuzzing"))]
mod tests {
    use crate::StateSyncMultiplexer;
    use consensus_notifications::new_consensus_notifier_listener_pair;
    use diem_config::{config::RocksdbConfig, utils::get_genesis_txn};
    use diem_crypto::HashValue;
    use diem_data_client::diemnet::DiemNetDataClient;
    use diem_genesis_tool::test_config;
    use diem_infallible::RwLock;
    use diem_temppath::TempPath;
    use diem_time_service::TimeService;
    use diem_types::{
        block_info::BlockInfo, ledger_info::LedgerInfo, on_chain_config::ON_CHAIN_CONFIG_REGISTRY,
        protocol_spec::DpnProto, waypoint::Waypoint,
    };
    use diem_vm::DiemVM;
    use diemdb::DiemDB;
    use event_notifications::EventSubscriptionService;
    use executor::Executor;
    use executor_test_helpers::bootstrap_genesis;
    use futures::{FutureExt, StreamExt};
    use mempool_notifications::new_mempool_notifier_listener_pair;
    use network::application::{interface::MultiNetworkSender, storage::PeerMetadataStorage};
    use std::{collections::HashMap, sync::Arc};
    use storage_interface::default_protocol::DbReaderWriter;
    use storage_service_client::StorageServiceClient;

    #[test]
    fn test_new_initialized_configs() {
        // Create a test database
        let tmp_dir = TempPath::new();
        let db = DiemDB::open(&tmp_dir, false, None, RocksdbConfig::default(), true).unwrap();
        let (_, db_rw) = DbReaderWriter::wrap(db);

        // Bootstrap the database
        let (node_config, _) = test_config();
        bootstrap_genesis::<DiemVM>(&db_rw, get_genesis_txn(&node_config).unwrap()).unwrap();

        // Create mempool and consensus notifiers
        let (mempool_notifier, _) = new_mempool_notifier_listener_pair();
        let (_, consensus_listener) = new_consensus_notifier_listener_pair(0);

        // Create the event subscription service and a reconfig subscriber
        let mut event_subscription_service = EventSubscriptionService::new(
            ON_CHAIN_CONFIG_REGISTRY,
            Arc::new(RwLock::new(db_rw.clone())),
        );
        let mut reconfiguration_subscriber = event_subscription_service
            .subscribe_to_reconfigurations()
            .unwrap();

        // Create a test diem data client
        let network_client = StorageServiceClient::new(
            MultiNetworkSender::new(HashMap::new()),
            PeerMetadataStorage::new(&[]),
        );
        let (diem_data_client, _) = DiemNetDataClient::new(
            node_config.state_sync.storage_service,
            TimeService::mock(),
            network_client,
        );

        // Create the multiplexer
        let _ = StateSyncMultiplexer::new(
            vec![],
            mempool_notifier,
            consensus_listener,
            db_rw.clone(),
            Box::new(Executor::<DpnProto, DiemVM>::new(db_rw)),
            &node_config,
            Waypoint::new_any(&LedgerInfo::new(BlockInfo::empty(), HashValue::random())),
            event_subscription_service,
            diem_data_client,
        );

        // Verify the initial configs were notified
        assert!(reconfiguration_subscriber
            .select_next_some()
            .now_or_never()
            .is_some());
    }
}
