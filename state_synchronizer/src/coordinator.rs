// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{counters, downloader::FetchChunkMsg};
use config::config::NodeConfig;
use execution_proto::proto::{
    execution::{ExecuteChunkRequest, ExecuteChunkResponse},
    execution_grpc::ExecutionClient,
};
use failure::prelude::*;
use futures::{
    channel::{mpsc, oneshot},
    Future, FutureExt, SinkExt, StreamExt,
};
use grpc_helpers::convert_grpc_response;
use grpcio::{ChannelBuilder, EnvBuilder};
use logger::prelude::*;
use nextgen_crypto::*;
use proto_conv::IntoProto;
use std::{collections::BTreeMap, pin::Pin, sync::Arc};
use storage_client::{StorageRead, StorageReadServiceClient};
use types::{ledger_info::LedgerInfoWithSignatures, proto::transaction::TransactionListWithProof};

/// unified message used for communication with Coordinator
// TODO: remove lint whitelist
#[allow(clippy::large_enum_variant)]
pub enum CoordinatorMsg<Sig> {
    // sent from client to initiate new sync
    Requested(LedgerInfoWithSignatures<Sig>, oneshot::Sender<SyncStatus>),
    // sent from client to notify about new txn commit
    Commit(u64),
    // is sent from Downloader to Coordinator to indicate that new batch is ready
    Fetched(
        Result<TransactionListWithProof>,
        LedgerInfoWithSignatures<Sig>,
    ),
}

#[derive(Clone, Debug, PartialEq)]
pub enum SyncStatus {
    Finished,
    ExecutionFailed,
    StorageReadFailed,
    DownloadFailed,
    DownloaderNotAvailable,
    ChunkIsEmpty,
}

/// used to coordinate synchronization process
/// handles Consensus requests and drives sync with remote peers
pub struct SyncCoordinator<T, Sig> {
    // communication with SyncCoordinator is done via this channel
    receiver: mpsc::UnboundedReceiver<CoordinatorMsg<Sig>>,
    // connection to transaction fetcher
    sender_to_downloader: mpsc::Sender<FetchChunkMsg<Sig>>,

    // last committed version that validator is aware of
    known_version: u64,
    // target state to sync to
    target: Option<LedgerInfoWithSignatures<Sig>>,
    // used to track progress of synchronization
    sync_position: u64,
    // subscribers of synchronization
    // each of them will be notified once their target version is ready
    subscribers: BTreeMap<u64, Vec<oneshot::Sender<SyncStatus>>>,
    executor_proxy: T,
}

impl<T: ExecutorProxyTrait, Sig: Signature> SyncCoordinator<T, Sig> {
    pub fn new(
        receiver: mpsc::UnboundedReceiver<CoordinatorMsg<Sig>>,
        sender_to_downloader: mpsc::Sender<FetchChunkMsg<Sig>>,
        executor_proxy: T,
    ) -> Self {
        Self {
            receiver,
            sender_to_downloader,

            known_version: 0,
            target: None,
            sync_position: 0,
            subscribers: BTreeMap::new(),
            executor_proxy,
        }
    }

    /// main routine. starts sync coordinator that listens for CoordinatorMsg
    pub async fn start(mut self) {
        while let Some(msg) = self.receiver.next().await {
            match msg {
                CoordinatorMsg::Requested(target, subscriber) => {
                    self.handle_request(target, subscriber).await;
                }
                CoordinatorMsg::Fetched(Ok(txn_list_with_proof), ledger_info_with_sigs) => {
                    self.process_transactions(txn_list_with_proof, ledger_info_with_sigs)
                        .await;
                }
                CoordinatorMsg::Fetched(Err(_), _) => {
                    self.notify_subscribers(SyncStatus::DownloadFailed);
                }
                CoordinatorMsg::Commit(version) => {
                    self.handle_commit(version);
                }
            }
        }
    }

    fn target_version(&self) -> u64 {
        match &self.target {
            Some(target) => target.ledger_info().version(),
            None => 0,
        }
    }

    /// Consensus request handler
    async fn handle_request(
        &mut self,
        target: LedgerInfoWithSignatures<Sig>,
        subscriber: oneshot::Sender<SyncStatus>,
    ) {
        let requested_version = target.ledger_info().version();
        let committed_version = self.executor_proxy.get_latest_version().await;

        // if requested version equals to current committed, just pass ledger info to executor
        // there might be still empty blocks between committed state and requested
        if let Ok(version) = committed_version {
            if version == requested_version {
                let status = match self
                    .store_transactions(TransactionListWithProof::new(), target)
                    .await
                {
                    Ok(_) => SyncStatus::Finished,
                    Err(_) => SyncStatus::ExecutionFailed,
                };
                if subscriber.send(status).is_err() {
                    log_collector_error!(
                        "[state synchronizer] coordinator failed to notify subscriber"
                    );
                }
                return;
            }
        }

        if requested_version > self.target_version() {
            self.target = Some(target.clone());
        }

        self.subscribers
            .entry(requested_version)
            .or_insert_with(|| vec![])
            .push(subscriber);

        if self.sync_position == 0 {
            // start new fetch
            match committed_version {
                Ok(version) => {
                    self.known_version = version;
                    self.sync_position = self.known_version + 1;
                    // send request to Downloader
                    let fetch_request = FetchChunkMsg {
                        start_version: self.sync_position,
                        target,
                    };
                    if self.sender_to_downloader.send(fetch_request).await.is_err() {
                        self.notify_subscribers(SyncStatus::DownloaderNotAvailable);
                    }
                }
                Err(_) => {
                    self.notify_subscribers(SyncStatus::StorageReadFailed);
                }
            }
        }
    }

    /// processes batch of transactions downloaded by fetcher
    /// executes transactions, updates progress state, notifies subscribers if some sync is finished
    async fn process_transactions(
        &mut self,
        txn_list_with_proof: TransactionListWithProof,
        target: LedgerInfoWithSignatures<Sig>,
    ) {
        let chunk_size = txn_list_with_proof.get_transactions().len() as u64;
        if chunk_size == 0 {
            self.notify_subscribers(SyncStatus::ChunkIsEmpty);
        }
        self.sync_position += chunk_size;

        if let Some(target) = self.target.clone() {
            if self.sync_position <= self.target_version() {
                let fetch_msg = FetchChunkMsg {
                    start_version: self.sync_position,
                    target,
                };
                // start download of next batch
                if self.sender_to_downloader.send(fetch_msg).await.is_err() {
                    self.notify_subscribers(SyncStatus::DownloaderNotAvailable);
                    return;
                }
            }
        }

        let status = match self.store_transactions(txn_list_with_proof, target).await {
            Ok(_) => SyncStatus::Finished,
            Err(_) => SyncStatus::ExecutionFailed,
        };
        counters::STATE_SYNC_TXN_REPLAYED.inc_by(chunk_size as i64);
        self.notify_subscribers(status);
    }

    fn notify_subscribers(&mut self, result: SyncStatus) {
        let mut active_subscribers = match result {
            SyncStatus::Finished => self.subscribers.split_off(&self.sync_position),
            _ => BTreeMap::new(),
        };

        // notify subscribers if some syncs are ready
        for channels in self.subscribers.values_mut() {
            channels.drain(..).for_each(|ch| {
                if ch.send(result.clone()).is_err() {
                    log_collector_error!(
                        "[state synchronizer] coordinator failed to notify subscriber"
                    );
                }
            });
        }
        self.subscribers.clear();
        self.subscribers.append(&mut active_subscribers);
        // reset sync state if done
        if self.subscribers.is_empty() {
            self.sync_position = 0;
        }
    }

    async fn store_transactions(
        &self,
        txn_list_with_proof: TransactionListWithProof,
        target: LedgerInfoWithSignatures<Sig>,
    ) -> Result<ExecuteChunkResponse> {
        let mut req = ExecuteChunkRequest::new();
        req.set_txn_list_with_proof(txn_list_with_proof);
        req.set_ledger_info_with_sigs(target.clone().into_proto());
        self.executor_proxy.execute_chunk(req).await
    }

    fn handle_commit(&self, _version: u64) {
        // TODO: add actual handler
    }
}

/// Proxy execution for state synchronization
pub trait ExecutorProxyTrait: Sync + Send {
    /// Return the latest known version
    fn get_latest_version(&self) -> Pin<Box<dyn Future<Output = Result<u64>> + Send>>;

    /// Execute and commit a batch of transactions
    fn execute_chunk(
        &self,
        request: ExecuteChunkRequest,
    ) -> Pin<Box<dyn Future<Output = Result<ExecuteChunkResponse>> + Send>>;
}

pub(crate) struct ExecutorProxy {
    storage_client: Arc<StorageReadServiceClient>,
    execution_client: Arc<ExecutionClient>,
}

impl ExecutorProxy {
    pub fn new(config: &NodeConfig) -> Self {
        let connection_str = format!("localhost:{}", config.execution.port);
        let env = Arc::new(EnvBuilder::new().name_prefix("grpc-coord-").build());
        let execution_client = Arc::new(ExecutionClient::new(
            ChannelBuilder::new(Arc::clone(&env)).connect(&connection_str),
        ));
        let storage_client = Arc::new(StorageReadServiceClient::new(
            env,
            &config.storage.address,
            config.storage.port,
        ));
        Self {
            storage_client,
            execution_client,
        }
    }
}

impl ExecutorProxyTrait for ExecutorProxy {
    fn get_latest_version(&self) -> Pin<Box<dyn Future<Output = Result<u64>> + Send>> {
        let client = Arc::clone(&self.storage_client);
        async move {
            let resp = client.update_to_latest_ledger_async(0, vec![]).await?;
            Ok(resp.1.ledger_info().version())
        }
            .boxed()
    }

    fn execute_chunk(
        &self,
        request: ExecuteChunkRequest,
    ) -> Pin<Box<dyn Future<Output = Result<ExecuteChunkResponse>> + Send>> {
        let client = Arc::clone(&self.execution_client);
        convert_grpc_response(client.execute_chunk_async(&request)).boxed()
    }
}
