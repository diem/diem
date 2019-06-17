// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    chained_bft::QuorumCert,
    state_synchronizer::{
        coordinator::{
            CoordinatorMsg, ExecutorProxy, ExecutorProxyTrait, SyncCoordinator, SyncStatus,
        },
        downloader::Downloader,
    },
};
use config::config::NodeConfig;
use failure::prelude::*;
use futures::{
    channel::{mpsc, oneshot},
    future::{Future, FutureExt, TryFutureExt},
    SinkExt,
};
use grpcio::EnvBuilder;
use logger::prelude::*;
use network::validator_network::ConsensusNetworkSender;
use std::sync::Arc;
use storage_client::{StorageRead, StorageReadServiceClient};
use tokio::runtime::TaskExecutor;
use types::transaction::TransactionListWithProof;

/// Used for synchronization between validators for committed states
pub struct StateSynchronizer {
    synchronizer_to_coordinator: mpsc::UnboundedSender<CoordinatorMsg>,
    storage_read_client: Arc<StorageRead>,
}

impl StateSynchronizer {
    /// Setup state synchronizer. spawns coordinator and downloader routines on executor
    pub fn new<E: ExecutorProxyTrait + 'static>(
        network: ConsensusNetworkSender,
        executor: TaskExecutor,
        config: &NodeConfig,
        executor_proxy: E,
    ) -> Self {
        let (coordinator_sender, coordinator_receiver) = mpsc::unbounded();
        let (fetcher_sender, fetcher_receiver) = mpsc::channel(1);

        let coordinator =
            SyncCoordinator::new(coordinator_receiver, fetcher_sender, executor_proxy);
        let downloader = Downloader::new(
            fetcher_receiver,
            coordinator_sender.clone(),
            network,
            config.base.node_sync_batch_size,
            config.base.node_sync_retries,
        );

        executor.spawn(coordinator.start().boxed().unit_error().compat());
        executor.spawn(downloader.start().boxed().unit_error().compat());

        let env = Arc::new(EnvBuilder::new().name_prefix("grpc-sync-").build());
        let storage_read_client = Arc::new(StorageReadServiceClient::new(
            env,
            &config.storage.address,
            config.storage.port,
        ));

        Self {
            synchronizer_to_coordinator: coordinator_sender,
            storage_read_client,
        }
    }

    /// Sync validator's state up to given `version`
    pub fn sync_to(&self, qc: QuorumCert) -> impl Future<Output = Result<SyncStatus>> {
        let mut sender = self.synchronizer_to_coordinator.clone();
        let (cb_sender, cb_receiver) = oneshot::channel();
        async move {
            sender
                .send(CoordinatorMsg::Requested(qc, cb_sender))
                .await?;
            let sync_status = cb_receiver.await?;
            Ok(sync_status)
        }
    }

    /// Get a batch of transactions
    pub fn get_chunk(
        &self,
        start_version: u64,
        target_version: u64,
        batch_size: u64,
    ) -> impl Future<Output = Result<TransactionListWithProof>> {
        let client = Arc::clone(&self.storage_read_client);
        async move {
            let txn_list_with_proof = client
                .get_transactions_async(
                    start_version,
                    batch_size,
                    target_version,
                    false, /* fetch_events */
                )
                .await?;

            if txn_list_with_proof.transaction_and_infos.is_empty() {
                log_collector_warn!(
                    "Not able to get txn from version {} for {} items",
                    start_version,
                    batch_size
                );
            }
            Ok(txn_list_with_proof)
        }
    }
}

/// Make the state synchronizer
pub fn setup_state_synchronizer(
    network: ConsensusNetworkSender,
    executor: TaskExecutor,
    config: &NodeConfig,
) -> StateSynchronizer {
    let executor_proxy = ExecutorProxy::new(config);
    StateSynchronizer::new(network, executor, config, executor_proxy)
}
