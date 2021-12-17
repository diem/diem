// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    driver::{DriverConfiguration, StateSyncDriver},
    driver_client::{ClientNotificationListener, DriverClient, DriverNotification},
    notification_handlers::{ConsensusNotificationHandler, MempoolNotificationHandler},
    storage_synchronizer::StorageSynchronizer,
};
use consensus_notifications::ConsensusNotificationListener;
use data_streaming_service::streaming_client::StreamingServiceClient;
use diem_config::config::NodeConfig;
use diem_data_client::diemnet::DiemNetDataClient;
use diem_infallible::RwLock;
use diem_types::waypoint::Waypoint;
use event_notifications::EventSubscriptionService;
use executor_types::ChunkExecutorTrait;
use futures::channel::mpsc;
use mempool_notifications::MempoolNotificationSender;
use std::sync::Arc;
use storage_interface::DbReaderWriter;
use tokio::runtime::{Builder, Runtime};

/// Creates a new state sync driver and client
pub struct DriverFactory {
    _driver_runtime: Option<Runtime>,
    client_notification_sender: mpsc::UnboundedSender<DriverNotification>,
}

impl DriverFactory {
    /// Creates and spawns a new state sync driver
    pub fn create_and_spawn_driver<
        ChunkExecutor: ChunkExecutorTrait + 'static,
        MempoolNotifier: MempoolNotificationSender + 'static,
    >(
        create_runtime: bool,
        node_config: &NodeConfig,
        waypoint: Waypoint,
        storage: DbReaderWriter,
        chunk_executor: Arc<ChunkExecutor>,
        mempool_notification_sender: MempoolNotifier,
        consensus_listener: ConsensusNotificationListener,
        event_subscription_service: EventSubscriptionService,
        diem_data_client: DiemNetDataClient,
        streaming_service_client: StreamingServiceClient,
    ) -> Self {
        // Create the notification handlers
        let (client_notification_sender, client_notification_receiver) = mpsc::unbounded();
        let client_notification_listener =
            ClientNotificationListener::new(client_notification_receiver);
        let consensus_notification_handler = ConsensusNotificationHandler::new(consensus_listener);
        let mempool_notification_handler =
            MempoolNotificationHandler::new(mempool_notification_sender);

        // Create the driver configuration
        let driver_configuration = DriverConfiguration::new(
            node_config.state_sync.state_sync_driver,
            node_config.base.role,
            waypoint,
        );

        // Create a storage synchronizer
        let storage_synchronizer =
            StorageSynchronizer::new(chunk_executor, Arc::new(RwLock::new(storage)));

        // Create the driver
        let state_sync_driver = StateSyncDriver::new(
            client_notification_listener,
            consensus_notification_handler,
            driver_configuration,
            event_subscription_service,
            mempool_notification_handler,
            storage_synchronizer,
            diem_data_client,
            streaming_service_client,
        );

        // Spawn the driver
        let driver_runtime = if create_runtime {
            // Create a new runtime for the driver
            let driver_runtime = Builder::new_multi_thread()
                .thread_name("state-sync-driver")
                .enable_all()
                .build()
                .expect("Failed to create state sync v2 driver runtime!");
            driver_runtime.spawn(state_sync_driver.start_driver());
            Some(driver_runtime)
        } else {
            // Spawn the driver on the current runtime
            tokio::spawn(state_sync_driver.start_driver());
            None
        };

        Self {
            _driver_runtime: driver_runtime,
            client_notification_sender,
        }
    }

    /// Returns a new client that can be used to communicate with the driver
    pub fn create_driver_client(&self) -> DriverClient {
        DriverClient::new(self.client_notification_sender.clone())
    }
}
