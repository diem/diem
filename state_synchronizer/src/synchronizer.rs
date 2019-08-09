// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    coordinator::{CoordinatorMsg, ExecutorProxy, ExecutorProxyTrait, SyncCoordinator, SyncStatus},
    downloader::Downloader,
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
use tokio::runtime::{Builder, Runtime};
use types::{ledger_info::LedgerInfoWithSignatures, transaction::TransactionListWithProof};

pub struct StateSynchronizer {
    _runtime: Runtime,
    synchronizer_to_coordinator: mpsc::UnboundedSender<CoordinatorMsg>,
}

impl StateSynchronizer {
    /// Setup state synchronizer. spawns coordinator and downloader routines on executor
    pub fn bootstrap(network: ConsensusNetworkSender, config: &NodeConfig) -> Self {
        let executor_proxy = ExecutorProxy::new(config);
        Self::bootstrap_with_executor_proxy(network, config, executor_proxy)
    }

    pub fn bootstrap_with_executor_proxy<E: ExecutorProxyTrait + 'static>(
        // TODO: move to separate network stack
        network: ConsensusNetworkSender,
        config: &NodeConfig,
        executor_proxy: E,
    ) -> Self {
        let runtime = Builder::new()
            .name_prefix("state-sync-")
            .build()
            .expect("[state synchronizer] failed to create runtime");
        let executor = runtime.executor();

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

        Self {
            _runtime: runtime,
            synchronizer_to_coordinator: coordinator_sender,
        }
    }

    pub fn create_client(&self, config: &NodeConfig) -> Arc<StateSyncClient> {
        let env = Arc::new(EnvBuilder::new().name_prefix("grpc-sync-").build());
        let storage_read_client = Arc::new(StorageReadServiceClient::new(
            env,
            &config.storage.address,
            config.storage.port,
        ));

        Arc::new(StateSyncClient {
            coordinator_sender: self.synchronizer_to_coordinator.clone(),
            storage_read_client,
        })
    }
}

pub struct StateSyncClient {
    coordinator_sender: mpsc::UnboundedSender<CoordinatorMsg>,
    // TODO: temporary. get rid of it after move out of Consensus p2p stack
    storage_read_client: Arc<dyn StorageRead>,
}

impl StateSyncClient {
    /// Sync validator's state up to given `version`
    pub fn sync_to(
        &self,
        target: LedgerInfoWithSignatures,
    ) -> impl Future<Output = Result<SyncStatus>> {
        let mut sender = self.coordinator_sender.clone();
        let (cb_sender, cb_receiver) = oneshot::channel();
        async move {
            sender
                .send(CoordinatorMsg::Requested(target, cb_sender))
                .await?;
            let sync_status = cb_receiver.await?;
            Ok(sync_status)
        }
    }

    /// Notifies state synchronizer about new version
    pub fn commit(&self, version: u64) -> impl Future<Output = Result<()>> {
        let mut sender = self.coordinator_sender.clone();
        async move {
            sender.send(CoordinatorMsg::Commit(version)).await?;
            Ok(())
        }
    }

    /// Get a batch of transactions
    pub fn get_chunk(
        &self,
        start_version: u64,
        target_version: u64,
        batch_size: u64,
    ) -> impl Future<Output = Result<TransactionListWithProof>> {
        // TODO: shouldn't be part of a client. Remove it once we move out of Consensus p2p stack
        // handler should live in separate service component
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
