// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    chunk_request::{GetChunkRequest, TargetType},
    chunk_response::{GetChunkResponse, ResponseLedgerInfo},
    client::{CoordinatorMessage, SyncRequest},
    counters,
    error::Error,
    executor_proxy::ExecutorProxyTrait,
    logging::{LogEntry, LogEvent, LogSchema},
    network::{StateSyncEvents, StateSyncMessage, StateSyncSender},
    request_manager::RequestManager,
    shared_components::SyncState,
};
use diem_config::{
    config::{NodeConfig, PeerNetworkId, RoleType, StateSyncConfig},
    network_id::NodeNetworkId,
};
use diem_logger::prelude::*;
use diem_mempool::CommitResponse;
use diem_types::{
    contract_event::ContractEvent,
    ledger_info::LedgerInfoWithSignatures,
    transaction::{Transaction, TransactionListWithProof, Version},
    waypoint::Waypoint,
    PeerId,
};
use fail::fail_point;
use futures::{
    channel::{mpsc, oneshot},
    stream::select_all,
    StreamExt,
};
use mempool_notifications::MempoolNotificationSender;
use network::{protocols::network::Event, transport::ConnectionMetadata};
use std::{
    cmp,
    collections::HashMap,
    time::{Duration, SystemTime},
};
use tokio::time::interval;
use tokio_stream::wrappers::IntervalStream;

#[derive(Clone, Debug, PartialEq, Eq)]
struct PendingRequestInfo {
    expiration_time: SystemTime,
    known_version: u64,
    request_epoch: u64,
    target_li: Option<LedgerInfoWithSignatures>,
    chunk_limit: u64,
}

/// Coordination of the state sync process is driven by StateSyncCoordinator. The `start()`
/// function runs an infinite event loop and triggers actions based on external and internal
/// (local) requests. The coordinator works in two modes (depending on the role):
/// * FullNode: infinite stream of ChunkRequests is sent to the predefined static peers
/// (the parent is going to reply with a ChunkResponse if its committed version becomes
/// higher within the timeout interval).
/// * Validator: the ChunkRequests are generated on demand for a specific target LedgerInfo to
/// synchronize to.
pub(crate) struct StateSyncCoordinator<T, M> {
    // used to process client requests
    client_events: mpsc::UnboundedReceiver<CoordinatorMessage>,
    // used to send messages (e.g. notifications about newly committed txns) to mempool
    mempool_notifier: M,
    // Current state of the storage, which includes both the latest committed transaction and the
    // latest transaction covered by the LedgerInfo (see `SynchronizerState` documentation).
    // The state is updated via syncing with the local storage.
    local_state: SyncState,
    // config
    config: StateSyncConfig,
    // role of node
    role: RoleType,
    // An initial waypoint: for as long as the local version is less than a version determined by
    // waypoint a node is not going to be abl
    waypoint: Waypoint,
    // Actor for sending chunk requests
    // Manages to whom and how to send chunk requests
    request_manager: RequestManager,
    // Optional sync request to be called when the target sync is reached
    sync_request: Option<SyncRequest>,
    // If we're a full node syncing to the latest state, this holds the highest ledger info
    // we know about and are currently syncing to. This allows us to incrementally sync to
    // ledger infos in storage. Higher ledger infos will only be considered once we sync to this.
    target_ledger_info: Option<LedgerInfoWithSignatures>,
    // Option initialization listener to be called when the coordinator is caught up with
    // its waypoint.
    initialization_listener: Option<oneshot::Sender<Result<(), Error>>>,
    // queue of incoming long polling requests
    // peer will be notified about new chunk of transactions if it's available before expiry time
    subscriptions: HashMap<PeerNetworkId, PendingRequestInfo>,
    executor_proxy: T,
}

impl<T: ExecutorProxyTrait, M: MempoolNotificationSender> StateSyncCoordinator<T, M> {
    pub fn new(
        client_events: mpsc::UnboundedReceiver<CoordinatorMessage>,
        mempool_notifier: M,
        network_senders: HashMap<NodeNetworkId, StateSyncSender>,
        node_config: &NodeConfig,
        waypoint: Waypoint,
        executor_proxy: T,
        initial_state: SyncState,
    ) -> Result<Self, Error> {
        info!(LogSchema::event_log(LogEntry::Waypoint, LogEvent::Initialize).waypoint(waypoint));

        // Create a new request manager.
        let role = node_config.base.role;
        let tick_interval_ms = node_config.state_sync.tick_interval_ms;
        let retry_timeout_val = match role {
            RoleType::FullNode => tick_interval_ms
                .checked_add(node_config.state_sync.long_poll_timeout_ms)
                .ok_or_else(|| {
                    Error::IntegerOverflow("Fullnode retry timeout has overflown!".into())
                })?,
            RoleType::Validator => tick_interval_ms.checked_mul(2).ok_or_else(|| {
                Error::IntegerOverflow("Validator retry timeout has overflown!".into())
            })?,
        };
        let request_manager = RequestManager::new(
            Duration::from_millis(retry_timeout_val),
            Duration::from_millis(node_config.state_sync.multicast_timeout_ms),
            network_senders,
        );

        Ok(Self {
            client_events,
            mempool_notifier,
            local_state: initial_state,
            config: node_config.state_sync.clone(),
            role,
            waypoint,
            request_manager,
            subscriptions: HashMap::new(),
            sync_request: None,
            target_ledger_info: None,
            initialization_listener: None,
            executor_proxy,
        })
    }

    /// main routine. starts sync coordinator that listens for CoordinatorMsg
    pub async fn start(
        mut self,
        network_handles: Vec<(NodeNetworkId, StateSyncSender, StateSyncEvents)>,
    ) {
        info!(LogSchema::new(LogEntry::RuntimeStart));
        let mut interval = IntervalStream::new(interval(Duration::from_millis(
            self.config.tick_interval_ms,
        )))
        .fuse();

        let events: Vec<_> = network_handles
            .into_iter()
            .map(|(network_id, _sender, events)| events.map(move |e| (network_id.clone(), e)))
            .collect();
        let mut network_events = select_all(events).fuse();

        loop {
            let _timer = counters::MAIN_LOOP.start_timer();
            ::futures::select! {
                msg = self.client_events.select_next_some() => {
                    match msg {
                        CoordinatorMessage::SyncRequest(request) => {
                            let _timer = counters::PROCESS_COORDINATOR_MSG_LATENCY
                                .with_label_values(&[counters::SYNC_MSG_LABEL])
                                .start_timer();
                            if let Err(e) = self.process_sync_request(*request) {
                                error!(LogSchema::new(LogEntry::SyncRequest).error(&e));
                                counters::SYNC_REQUEST_RESULT.with_label_values(&[counters::FAIL_LABEL]).inc();
                            }
                        }
                        CoordinatorMessage::CommitNotification(notification) => {
                            let _timer = counters::PROCESS_COORDINATOR_MSG_LATENCY
                                .with_label_values(&[counters::COMMIT_MSG_LABEL])
                                .start_timer();
                            if let Err(e) = self.process_commit_notification(notification.committed_transactions, Some(notification.callback), notification.reconfiguration_events, None).await {
                                counters::CONSENSUS_COMMIT_FAIL_COUNT.inc();
                                error!(LogSchema::event_log(LogEntry::ConsensusCommit, LogEvent::PostCommitFail).error(&e));
                            }
                        }
                        CoordinatorMessage::GetSyncState(callback) => {
                            let _ = self.get_sync_state(callback);
                        }
                        CoordinatorMessage::WaitForInitialization(cb_sender) => {
                            if let Err(e) = self.wait_for_initialization(cb_sender) {
                                error!(LogSchema::new(LogEntry::Waypoint).error(&e));
                            }
                        }
                    };
                },
                (network_id, event) = network_events.select_next_some() => {
                    match event {
                        Event::NewPeer(metadata) => {
                            if let Err(e) = self.process_new_peer(network_id, metadata) {
                                error!(LogSchema::new(LogEntry::NewPeer).error(&e));
                            }
                        }
                        Event::LostPeer(metadata) => {
                            if let Err(e) = self.process_lost_peer(network_id, metadata.remote_peer_id) {
                                error!(LogSchema::new(LogEntry::LostPeer).error(&e));
                            }
                        }
                        Event::Message(peer_id, message) => {
                            if let Err(e) = self.process_chunk_message(network_id.clone(), peer_id, message).await {
                                error!(LogSchema::new(LogEntry::ProcessChunkMessage).error(&e));
                            }
                        }
                        unexpected_event => {
                            counters::NETWORK_ERROR_COUNT.inc();
                            warn!(LogSchema::new(LogEntry::NetworkError),
                            "received unexpected network event: {:?}", unexpected_event);
                        },

                    }
                },
                _ = interval.select_next_some() => {
                    if let Err(e) = self.check_progress() {
                        error!(LogSchema::event_log(LogEntry::ProgressCheck, LogEvent::Fail).error(&e));
                    }
                }
            }
        }
    }

    fn process_new_peer(
        &mut self,
        network_id: NodeNetworkId,
        metadata: ConnectionMetadata,
    ) -> Result<(), Error> {
        let peer = PeerNetworkId(network_id, metadata.remote_peer_id);
        self.request_manager.enable_peer(peer, metadata)?;
        self.check_progress()
    }

    fn process_lost_peer(
        &mut self,
        network_id: NodeNetworkId,
        peer_id: PeerId,
    ) -> Result<(), Error> {
        let peer = PeerNetworkId(network_id, peer_id);
        self.request_manager.disable_peer(&peer)
    }

    pub(crate) async fn process_chunk_message(
        &mut self,
        network_id: NodeNetworkId,
        peer_id: PeerId,
        msg: StateSyncMessage,
    ) -> Result<(), Error> {
        let peer = PeerNetworkId(network_id, peer_id);
        match msg {
            StateSyncMessage::GetChunkRequest(request) => {
                // Time request handling
                let _timer = counters::PROCESS_MSG_LATENCY
                    .with_label_values(&[
                        &peer.raw_network_id().to_string(),
                        &peer.peer_id().to_string(),
                        counters::CHUNK_REQUEST_MSG_LABEL,
                    ])
                    .start_timer();

                // Process chunk request
                let process_result = self.process_chunk_request(peer.clone(), *request.clone());
                if let Err(ref error) = process_result {
                    error!(
                        LogSchema::event_log(LogEntry::ProcessChunkRequest, LogEvent::Fail)
                            .peer(&peer)
                            .error(&error.clone())
                            .local_li_version(self.local_state.committed_version())
                            .chunk_request(*request)
                    );
                    counters::PROCESS_CHUNK_REQUEST_COUNT
                        .with_label_values(&[
                            &peer.raw_network_id().to_string(),
                            &peer.peer_id().to_string(),
                            counters::FAIL_LABEL,
                        ])
                        .inc();
                } else {
                    counters::PROCESS_CHUNK_REQUEST_COUNT
                        .with_label_values(&[
                            &peer.raw_network_id().to_string(),
                            &peer.peer_id().to_string(),
                            counters::SUCCESS_LABEL,
                        ])
                        .inc();
                }
                process_result
            }
            StateSyncMessage::GetChunkResponse(response) => {
                // Time response handling
                let _timer = counters::PROCESS_MSG_LATENCY
                    .with_label_values(&[
                        &peer.raw_network_id().to_string(),
                        &peer.peer_id().to_string(),
                        counters::CHUNK_RESPONSE_MSG_LABEL,
                    ])
                    .start_timer();

                // Process chunk response
                self.process_chunk_response(&peer, *response).await
            }
        }
    }

    /// Sync up coordinator state with the local storage
    /// and updates the pending ledger info accordingly
    fn sync_state_with_local_storage(&mut self) -> Result<(), Error> {
        let new_state = self.executor_proxy.get_local_storage_state().map_err(|e| {
            counters::STORAGE_READ_FAIL_COUNT.inc();
            e
        })?;
        if new_state.trusted_epoch() > self.local_state.trusted_epoch() {
            info!(LogSchema::new(LogEntry::EpochChange)
                .old_epoch(self.local_state.trusted_epoch())
                .new_epoch(new_state.trusted_epoch()));
        }
        self.local_state = new_state;
        Ok(())
    }

    /// Verify that the local state's latest LI version (i.e. committed version) has reached the waypoint version.
    fn is_initialized(&self) -> bool {
        self.waypoint.version() <= self.local_state.committed_version()
    }

    fn wait_for_initialization(
        &mut self,
        cb_sender: oneshot::Sender<Result<(), Error>>,
    ) -> Result<(), Error> {
        if self.is_initialized() {
            Self::send_initialization_callback(cb_sender)?;
        } else {
            self.initialization_listener = Some(cb_sender);
        }

        Ok(())
    }

    /// This method requests state sync to sync to the target specified by the SyncRequest.
    /// If there is an existing sync request it will be overridden.
    /// Note: when processing a sync request, state sync assumes that it's the only one
    /// modifying storage, i.e., consensus is not trying to commit transactions concurrently.
    fn process_sync_request(&mut self, request: SyncRequest) -> Result<(), Error> {
        fail_point!("state_sync_v1::process_sync_request_message", |_| {
            Err(crate::error::Error::UnexpectedError(
                "Injected error in process_sync_request_message".into(),
            ))
        });

        // Full nodes don't support sync requests
        if self.role == RoleType::FullNode {
            return Err(Error::FullNodeSyncRequest);
        }

        let local_li_version = self.local_state.committed_version();
        let target_version = request.target.ledger_info().version();
        info!(
            LogSchema::event_log(LogEntry::SyncRequest, LogEvent::Received)
                .target_version(target_version)
                .local_li_version(local_li_version)
        );

        self.sync_state_with_local_storage()?;
        if !self.is_initialized() {
            return Err(Error::UninitializedError(
                "Unable to process sync request message!".into(),
            ));
        }

        if target_version == local_li_version {
            return Self::send_sync_req_callback(request, Ok(()));
        }
        if target_version < local_li_version {
            Self::send_sync_req_callback(
                request,
                Err(Error::UnexpectedError("Sync request to old version".into())),
            )?;
            return Err(Error::OldSyncRequestVersion(
                target_version,
                local_li_version,
            ));
        }

        // Save the new sync request
        self.sync_request = Some(request);

        // Send a chunk request for the sync target
        let known_version = self.local_state.synced_version();
        self.send_chunk_request_with_target(
            known_version,
            self.local_state.trusted_epoch(),
            self.create_sync_request_chunk_target(known_version)?,
        )
    }

    /// Notifies consensus of the given commit response.
    /// Note: if a callback is not specified, the response isn't sent anywhere.
    fn notify_consensus_of_commit_response(
        &self,
        commit_response: CommitResponse,
        callback: Option<oneshot::Sender<Result<CommitResponse, Error>>>,
    ) -> Result<(), Error> {
        if let Some(callback) = callback {
            if let Err(error) = callback.send(Ok(commit_response)) {
                counters::COMMIT_FLOW_FAIL
                    .with_label_values(&[counters::CONSENSUS_LABEL])
                    .inc();
                return Err(Error::CallbackSendFailed(format!(
                    "Failed to send commit ACK to consensus!: {:?}",
                    error
                )));
            }
        }
        Ok(())
    }

    /// This method updates state sync to process new transactions that have been committed
    /// to storage (e.g., through consensus or through a chunk response).
    /// When notified about a new commit we should: (i) respond to relevant long poll requests;
    /// (ii) update local sync and initialization requests (where appropriate); and (iii) publish
    /// on chain config updates.
    async fn process_commit_notification(
        &mut self,
        committed_transactions: Vec<Transaction>,
        commit_callback: Option<oneshot::Sender<Result<CommitResponse, Error>>>,
        reconfiguration_events: Vec<ContractEvent>,
        chunk_sender: Option<&PeerNetworkId>,
    ) -> Result<(), Error> {
        // We choose to re-sync the state with the storage as it's the simplest approach:
        // in case the performance implications of re-syncing upon every commit are high,
        // it's possible to manage some of the highest known versions in memory.
        self.sync_state_with_local_storage()?;
        self.update_sync_state_metrics_and_logs()?;

        // Notify mempool of commit
        let commit_response = match self
            .notify_mempool_of_committed_transactions(committed_transactions)
            .await
        {
            Ok(()) => CommitResponse::success(),
            Err(error) => {
                error!(LogSchema::new(LogEntry::CommitFlow).error(&error));
                CommitResponse::error(error.to_string())
            }
        };

        // Notify consensus of the commit response
        if let Err(error) =
            self.notify_consensus_of_commit_response(commit_response, commit_callback)
        {
            error!(LogSchema::new(LogEntry::CommitFlow).error(&error),);
        }

        // Check long poll subscriptions, update peer requests and sync request last progress
        // timestamp.
        self.check_subscriptions();
        let synced_version = self.local_state.synced_version();
        self.request_manager.remove_requests(synced_version);
        if let Some(peer) = chunk_sender {
            self.request_manager.process_success_response(peer);
        }
        if let Some(mut req) = self.sync_request.as_mut() {
            req.last_commit_timestamp = SystemTime::now();
        }

        // Check if we're now initialized or if we hit the sync request target
        self.check_initialized_or_sync_request_completed(synced_version)?;

        // Publish the on chain config updates
        if let Err(error) = self
            .executor_proxy
            .publish_on_chain_config_updates(reconfiguration_events)
        {
            counters::RECONFIG_PUBLISH_COUNT
                .with_label_values(&[counters::FAIL_LABEL])
                .inc();
            error!(LogSchema::event_log(LogEntry::Reconfig, LogEvent::Fail).error(&error));
        }

        Ok(())
    }

    /// Checks if we are now at the initialization point (i.e., the waypoint), or at the version
    /// specified by a sync request made by consensus.
    fn check_initialized_or_sync_request_completed(
        &mut self,
        synced_version: u64,
    ) -> Result<(), Error> {
        let committed_version = self.local_state.committed_version();
        let local_epoch = self.local_state.trusted_epoch();

        // Check if we're now initialized
        if self.is_initialized() {
            if let Some(initialization_listener) = self.initialization_listener.take() {
                info!(LogSchema::event_log(LogEntry::Waypoint, LogEvent::Complete)
                    .local_li_version(committed_version)
                    .local_synced_version(synced_version)
                    .local_epoch(local_epoch));
                Self::send_initialization_callback(initialization_listener)?;
            }
        }

        // Check if we're now at the sync request target
        if let Some(sync_request) = self.sync_request.as_ref() {
            let sync_target_version = sync_request.target.ledger_info().version();
            if synced_version > sync_target_version {
                return Err(Error::SyncedBeyondTarget(
                    synced_version,
                    sync_target_version,
                ));
            }
            if synced_version == sync_target_version {
                info!(
                    LogSchema::event_log(LogEntry::SyncRequest, LogEvent::Complete)
                        .local_li_version(committed_version)
                        .local_synced_version(synced_version)
                        .local_epoch(local_epoch)
                );
                counters::SYNC_REQUEST_RESULT
                    .with_label_values(&[counters::COMPLETE_LABEL])
                    .inc();
                if let Some(sync_request) = self.sync_request.take() {
                    Self::send_sync_req_callback(sync_request, Ok(()))?;
                }
            }
        }

        Ok(())
    }

    /// Notifies mempool that transactions have been committed.
    async fn notify_mempool_of_committed_transactions(
        &mut self,
        committed_transactions: Vec<Transaction>,
    ) -> Result<(), Error> {
        // Notify mempool of committed transactions
        let block_timestamp_usecs = self
            .local_state
            .committed_ledger_info()
            .ledger_info()
            .timestamp_usecs();
        if let Err(error) = self
            .mempool_notifier
            .notify_new_commit(
                committed_transactions,
                block_timestamp_usecs,
                self.config.mempool_commit_timeout_ms,
            )
            .await
        {
            counters::COMMIT_FLOW_FAIL
                .with_label_values(&[counters::MEMPOOL_LABEL])
                .inc();
            Err(error.into())
        } else {
            Ok(())
        }
    }

    /// Updates the metrics and logs based on the current (local) sync state.
    fn update_sync_state_metrics_and_logs(&mut self) -> Result<(), Error> {
        // Get data from local sync state
        let synced_version = self.local_state.synced_version();
        let committed_version = self.local_state.committed_version();
        let local_epoch = self.local_state.trusted_epoch();

        // Update versions
        counters::set_version(counters::VersionType::Synced, synced_version);
        counters::set_version(counters::VersionType::Committed, committed_version);
        counters::EPOCH.set(local_epoch as i64);

        // Update timestamps
        counters::set_timestamp(
            counters::TimestampType::Synced,
            self.executor_proxy.get_version_timestamp(synced_version)?,
        );
        counters::set_timestamp(
            counters::TimestampType::Committed,
            self.executor_proxy
                .get_version_timestamp(committed_version)?,
        );
        counters::set_timestamp(
            counters::TimestampType::Real,
            diem_infallible::duration_since_epoch().as_micros() as u64,
        );

        debug!(LogSchema::new(LogEntry::LocalState)
            .local_li_version(committed_version)
            .local_synced_version(synced_version)
            .local_epoch(local_epoch));
        Ok(())
    }

    /// Returns the current SyncState of state sync.
    /// Note: this is only used for testing and should be removed once integration/e2e tests
    /// are updated to not rely on this.
    fn get_sync_state(&mut self, callback: oneshot::Sender<SyncState>) -> Result<(), Error> {
        self.sync_state_with_local_storage()?;
        match callback.send(self.local_state.clone()) {
            Err(error) => Err(Error::CallbackSendFailed(format!(
                "Failed to get sync state! Error: {:?}",
                error
            ))),
            _ => Ok(()),
        }
    }

    /// There are two types of ChunkRequests:
    /// 1) Validator chunk requests are for a specific target LI and don't ask for long polling.
    /// 2) FullNode chunk requests don't specify a target LI and can allow long polling.
    fn process_chunk_request(
        &mut self,
        peer: PeerNetworkId,
        request: GetChunkRequest,
    ) -> Result<(), Error> {
        debug!(
            LogSchema::event_log(LogEntry::ProcessChunkRequest, LogEvent::Received)
                .peer(&peer)
                .chunk_request(request.clone())
                .local_li_version(self.local_state.committed_version())
        );
        fail_point!("state_sync_v1::process_chunk_request", |_| {
            Err(crate::error::Error::UnexpectedError(
                "Injected error in process_chunk_request".into(),
            ))
        });
        self.sync_state_with_local_storage()?;

        // Verify the chunk request is valid before trying to process it. If it's invalid,
        // penalize the peer's score.
        if let Err(error) = self.verify_chunk_request_is_valid(&request) {
            self.request_manager.process_invalid_chunk_request(&peer);
            return Err(error);
        }

        match request.target.clone() {
            TargetType::TargetLedgerInfo(li) => {
                self.process_request_for_target_and_highest(peer, request, Some(li), None)
            }
            TargetType::HighestAvailable {
                target_li,
                timeout_ms,
            } => self.process_request_for_target_and_highest(
                peer,
                request,
                target_li,
                Some(timeout_ms),
            ),
            TargetType::Waypoint(waypoint_version) => {
                self.process_request_for_waypoint(peer, request, waypoint_version)
            }
        }
    }

    fn verify_chunk_request_is_valid(&mut self, request: &GetChunkRequest) -> Result<(), Error> {
        // Ensure request versions are correctly formed
        if let Some(target_version) = request.target.version() {
            if target_version < request.known_version {
                return Err(Error::InvalidChunkRequest(
                    "Target version is less than known version! Discarding request.".into(),
                ));
            }
        }

        // Ensure request epochs are correctly formed
        if let Some(target_epoch) = request.target.epoch() {
            if target_epoch < request.current_epoch {
                return Err(Error::InvalidChunkRequest(
                    "Target epoch is less than current epoch! Discarding request.".into(),
                ));
            }
        }

        // Ensure the chunk limit is not zero
        if request.limit == 0 {
            return Err(Error::InvalidChunkRequest(
                "Chunk request limit is 0. Discarding request.".into(),
            ));
        }

        // Ensure the timeout is not zero
        if let TargetType::HighestAvailable {
            target_li: _,
            timeout_ms,
        } = request.target.clone()
        {
            if timeout_ms == 0 {
                return Err(Error::InvalidChunkRequest(
                    "Long poll timeout is 0. Discarding request.".into(),
                ));
            }
        }

        Ok(())
    }

    /// Processing requests with no target LedgerInfo (highest available) and potentially long
    /// polling.
    /// Assumes that the local state is uptodate with storage.
    fn process_request_for_target_and_highest(
        &mut self,
        peer: PeerNetworkId,
        request: GetChunkRequest,
        target_li: Option<LedgerInfoWithSignatures>,
        timeout_ms: Option<u64>,
    ) -> Result<(), Error> {
        let chunk_limit = std::cmp::min(request.limit, self.config.max_chunk_limit);
        let timeout = if let Some(timeout_ms) = timeout_ms {
            std::cmp::min(timeout_ms, self.config.max_timeout_ms)
        } else {
            self.config.max_timeout_ms
        };

        // If the node cannot respond to the request now (i.e., it's not up-to-date with the
        // requestor) add the request to the subscriptions to be handled when this node catches up.
        let local_version = self.local_state.committed_version();
        if local_version <= request.known_version {
            let expiration_time = SystemTime::now().checked_add(Duration::from_millis(timeout));
            if let Some(time) = expiration_time {
                let request_info = PendingRequestInfo {
                    expiration_time: time,
                    known_version: request.known_version,
                    request_epoch: request.current_epoch,
                    target_li,
                    chunk_limit,
                };
                self.subscriptions.insert(peer, request_info);
            }
            return Ok(());
        }

        let (target_li, highest_li) =
            self.calculate_target_and_highest_li(request.current_epoch, target_li, local_version)?;

        self.deliver_chunk(
            peer,
            request.known_version,
            ResponseLedgerInfo::ProgressiveLedgerInfo {
                target_li,
                highest_li,
            },
            chunk_limit,
        )
    }

    fn calculate_target_and_highest_li(
        &mut self,
        request_epoch: u64,
        request_target_li: Option<LedgerInfoWithSignatures>,
        local_version: u64,
    ) -> Result<(LedgerInfoWithSignatures, Option<LedgerInfoWithSignatures>), Error> {
        // If the request's epoch is in the past, `target_li` will be set to the end-of-epoch LI for that epoch
        let target_li = self.choose_response_li(request_epoch, request_target_li)?;

        let highest_li = if target_li.ledger_info().version() < local_version
            && target_li.ledger_info().epoch() == self.local_state.trusted_epoch()
        {
            // Only populate highest_li field if it's in the past, and the same epoch.
            // Recipient won't be able to verify ledger info if it's in a different epoch.
            Some(self.local_state.committed_ledger_info())
        } else {
            None
        };

        Ok((target_li, highest_li))
    }

    fn process_request_for_waypoint(
        &mut self,
        peer: PeerNetworkId,
        request: GetChunkRequest,
        waypoint_version: Version,
    ) -> Result<(), Error> {
        let mut limit = std::cmp::min(request.limit, self.config.max_chunk_limit);
        if self.local_state.committed_version() < waypoint_version {
            return Err(Error::UnexpectedError(format!(
                "Local version {} < requested waypoint version {}.",
                self.local_state.committed_version(),
                waypoint_version
            )));
        }
        if request.known_version >= waypoint_version {
            return Err(Error::UnexpectedError(format!(
                "Waypoint request version {} is not smaller than waypoint {}",
                request.known_version, waypoint_version
            )));
        }

        // Retrieve the waypoint LI.
        let waypoint_li = self
            .executor_proxy
            .get_epoch_ending_ledger_info(waypoint_version)?;

        // Txns are up to the end of request epoch with the proofs relative to the waypoint LI.
        let end_of_epoch_li = if waypoint_li.ledger_info().epoch() > request.current_epoch {
            let end_of_epoch_li = self
                .executor_proxy
                .get_epoch_change_ledger_info(request.current_epoch)?;
            if end_of_epoch_li.ledger_info().version() < request.known_version {
                return Err(Error::UnexpectedError(format!("Waypoint request's current_epoch (epoch {}, version {}) < waypoint request's known_version {}",
                                                          end_of_epoch_li.ledger_info().epoch(),
                                                          end_of_epoch_li.ledger_info().version(),
                                                          request.known_version,)));
            }
            let num_txns_until_end_of_epoch =
                end_of_epoch_li.ledger_info().version() - request.known_version;
            limit = std::cmp::min(limit, num_txns_until_end_of_epoch);
            Some(end_of_epoch_li)
        } else {
            None
        };

        self.deliver_chunk(
            peer,
            request.known_version,
            ResponseLedgerInfo::LedgerInfoForWaypoint {
                waypoint_li,
                end_of_epoch_li,
            },
            limit,
        )
    }

    /// Generate and send the ChunkResponse to the given peer.
    /// The chunk response contains transactions from the local storage with the proofs relative to
    /// the given target ledger info.
    /// In case target is None, the ledger info is set to the local highest ledger info.
    fn deliver_chunk(
        &mut self,
        peer: PeerNetworkId,
        known_version: u64,
        response_li: ResponseLedgerInfo,
        limit: u64,
    ) -> Result<(), Error> {
        let txns = self
            .executor_proxy
            .get_chunk(known_version, limit, response_li.version())?;
        let chunk_response = GetChunkResponse::new(response_li, txns);
        let log = LogSchema::event_log(LogEntry::ProcessChunkRequest, LogEvent::DeliverChunk)
            .chunk_response(chunk_response.clone())
            .peer(&peer);
        let msg = StateSyncMessage::GetChunkResponse(Box::new(chunk_response));
        let send_result = self.request_manager.send_chunk_response(&peer, msg);
        let send_result_label = if send_result.is_err() {
            counters::SEND_FAIL_LABEL
        } else {
            debug!(log);
            counters::SEND_SUCCESS_LABEL
        };
        counters::RESPONSES_SENT
            .with_label_values(&[
                &peer.raw_network_id().to_string(),
                &peer.peer_id().to_string(),
                send_result_label,
            ])
            .inc();

        send_result.map_err(|e| {
            error!(log.error(&e));
            Error::UnexpectedError(format!(
                "Network error in sending chunk response to {}",
                peer
            ))
        })
    }

    /// The choice of the LedgerInfo in the response follows the following logic:
    /// * response LI is either the requested target or the highest local LI if target is None.
    /// * if the response LI would not belong to `request_epoch`, change
    /// the response LI to the LI that is terminating `request_epoch`.
    fn choose_response_li(
        &self,
        request_epoch: u64,
        target: Option<LedgerInfoWithSignatures>,
    ) -> Result<LedgerInfoWithSignatures, Error> {
        let mut target_li = target.unwrap_or_else(|| self.local_state.committed_ledger_info());
        let target_epoch = target_li.ledger_info().epoch();
        if target_epoch > request_epoch {
            let end_of_epoch_li = self
                .executor_proxy
                .get_epoch_change_ledger_info(request_epoch)?;
            debug!(LogSchema::event_log(
                LogEntry::ProcessChunkRequest,
                LogEvent::PastEpochRequested
            )
            .old_epoch(request_epoch)
            .new_epoch(target_epoch));
            target_li = end_of_epoch_li;
        }
        Ok(target_li)
    }

    /// Applies (i.e., executes and stores) the chunk to storage iff `response` is valid.
    fn apply_chunk(
        &mut self,
        peer: &PeerNetworkId,
        response: GetChunkResponse,
    ) -> Result<(), Error> {
        debug!(
            LogSchema::event_log(LogEntry::ProcessChunkResponse, LogEvent::Received)
                .chunk_response(response.clone())
                .peer(peer)
        );
        fail_point!("state_sync_v1::apply_chunk", |_| {
            Err(crate::error::Error::UnexpectedError(
                "Injected error in apply_chunk".into(),
            ))
        });

        // Process the chunk based on the response type
        let txn_list_with_proof = response.txn_list_with_proof.clone();
        let chunk_size = response.txn_list_with_proof.len() as u64;
        let known_version = self.local_state.synced_version();
        match response.response_li {
            ResponseLedgerInfo::VerifiableLedgerInfo(li) => {
                self.process_response_with_target_and_highest(txn_list_with_proof, li, None)
            }
            ResponseLedgerInfo::ProgressiveLedgerInfo {
                target_li,
                highest_li,
            } => {
                let highest_li = highest_li.unwrap_or_else(|| target_li.clone());
                self.process_response_with_target_and_highest(
                    txn_list_with_proof,
                    target_li,
                    Some(highest_li),
                )
            }
            ResponseLedgerInfo::LedgerInfoForWaypoint {
                waypoint_li,
                end_of_epoch_li,
            } => self.process_response_with_waypoint_li(
                txn_list_with_proof,
                waypoint_li,
                end_of_epoch_li,
            ),
        }
        .map_err(|error| {
            self.request_manager.process_invalid_chunk(peer);
            Error::ProcessInvalidChunk(error.to_string())
        })?;

        // Update counters and logs with processed chunk information
        counters::STATE_SYNC_CHUNK_SIZE
            .with_label_values(&[
                &peer.raw_network_id().to_string(),
                &peer.peer_id().to_string(),
            ])
            .observe(chunk_size as f64);
        let new_version = known_version
            .checked_add(chunk_size)
            .ok_or_else(|| Error::IntegerOverflow("New version has overflown!".into()))?;
        debug!(
            LogSchema::event_log(LogEntry::ProcessChunkResponse, LogEvent::ApplyChunkSuccess),
            "Applied chunk of size {}. Previous version: {}, new version {}",
            chunk_size,
            known_version,
            new_version
        );

        // Log the request processing time (time from first requested until now).
        match self.request_manager.get_first_request_time(known_version) {
            None => {
                info!(
                    LogSchema::event_log(LogEntry::ProcessChunkResponse, LogEvent::ReceivedChunkWithoutRequest),
                    "Received a chunk of size {}, without making a request! Previous version: {}, new version {}",
                    chunk_size,
                    known_version,
                    new_version
                );
            }
            Some(first_request_time) => {
                if let Ok(duration) = SystemTime::now().duration_since(first_request_time) {
                    counters::SYNC_PROGRESS_DURATION.observe_duration(duration);
                }
            }
        }

        Ok(())
    }

    /// * Verifies, processes and stores the chunk in the given response.
    /// * Triggers post-commit actions based on new local state (after successfully processing a chunk).
    async fn process_chunk_response(
        &mut self,
        peer: &PeerNetworkId,
        response: GetChunkResponse,
    ) -> Result<(), Error> {
        // Ensure consensus isn't running, otherwise we might get a race with storage writes.
        if self.is_consensus_executing() {
            let error = Error::ConsensusIsExecuting;
            error!(LogSchema::new(LogEntry::ProcessChunkResponse,)
                .peer(peer)
                .error(&error));
            return Err(error);
        }

        // Verify the chunk response is well formed before trying to process it.
        self.verify_chunk_response_is_valid(peer, &response)?;

        // Validate the response and store the chunk if possible.
        // Any errors thrown here should be for detecting bad chunks.
        match self.apply_chunk(peer, response.clone()) {
            Ok(()) => {
                counters::APPLY_CHUNK_COUNT
                    .with_label_values(&[
                        &peer.raw_network_id().to_string(),
                        &peer.peer_id().to_string(),
                        counters::SUCCESS_LABEL,
                    ])
                    .inc();
            }
            Err(error) => {
                error!(LogSchema::event_log(
                    LogEntry::ProcessChunkResponse,
                    LogEvent::ApplyChunkFail
                )
                .peer(peer)
                .error(&error));
                counters::APPLY_CHUNK_COUNT
                    .with_label_values(&[
                        &peer.raw_network_id().to_string(),
                        &peer.peer_id().to_string(),
                        counters::FAIL_LABEL,
                    ])
                    .inc();
                return Err(error);
            }
        }

        // Process the newly committed chunk
        self.process_commit_notification(
            response.txn_list_with_proof.transactions.clone(),
            None,
            vec![],
            Some(peer),
        )
        .await
        .map_err(|error| {
            error!(
                LogSchema::event_log(LogEntry::ProcessChunkResponse, LogEvent::PostCommitFail)
                    .peer(peer)
                    .error(&error)
            );
            error
        })
    }

    fn verify_chunk_response_is_valid(
        &mut self,
        peer: &PeerNetworkId,
        response: &GetChunkResponse,
    ) -> Result<(), Error> {
        // Verify response comes from known peer
        if !self.request_manager.is_known_state_sync_peer(peer) {
            counters::RESPONSE_FROM_DOWNSTREAM_COUNT
                .with_label_values(&[
                    &peer.raw_network_id().to_string(),
                    &peer.peer_id().to_string(),
                ])
                .inc();
            self.request_manager.process_chunk_from_downstream(peer);
            return Err(Error::ReceivedChunkFromDownstream(peer.to_string()));
        }

        // Verify the chunk is not empty and that it starts at the correct version
        if let Some(first_chunk_version) = response.txn_list_with_proof.first_transaction_version {
            let known_version = self.local_state.synced_version();
            let expected_version = known_version
                .checked_add(1)
                .ok_or_else(|| Error::IntegerOverflow("Expected version has overflown!".into()))?;

            if first_chunk_version != expected_version {
                self.request_manager.process_chunk_version_mismatch(
                    peer,
                    first_chunk_version,
                    known_version,
                )?;
            }
        } else {
            // The chunk is empty
            self.request_manager.process_empty_chunk(peer);
            return Err(Error::ReceivedEmptyChunk(peer.to_string()));
        }

        // Verify the chunk has the expected type for the current syncing mode
        match &response.response_li {
            ResponseLedgerInfo::LedgerInfoForWaypoint {
                waypoint_li,
                end_of_epoch_li,
            } => self.verify_response_with_waypoint_li(waypoint_li, end_of_epoch_li),
            ResponseLedgerInfo::VerifiableLedgerInfo(response_li) => {
                self.verify_response_with_target_and_highest(response_li, &None)
            }
            ResponseLedgerInfo::ProgressiveLedgerInfo {
                target_li,
                highest_li,
            } => self.verify_response_with_target_and_highest(target_li, highest_li),
        }
    }

    fn verify_response_with_target_and_highest(
        &mut self,
        target_li: &LedgerInfoWithSignatures,
        highest_li: &Option<LedgerInfoWithSignatures>,
    ) -> Result<(), Error> {
        if !self.is_initialized() {
            return Err(Error::ReceivedWrongChunkType(
                "Received a progressive ledger info, but we're not initialized!".into(),
            ));
        }

        // If we're syncing to a specific target for consensus, valid responses
        // should not exceed the ledger info version of the sync request.
        if let Some(sync_request) = self.sync_request.as_ref() {
            let sync_request_version = sync_request.target.ledger_info().version();
            let response_version = target_li.ledger_info().version();
            if sync_request_version < response_version {
                let error_message = format!("Verifiable ledger info version is higher than the sync target. Received: {}, requested: {}.",
                                            response_version,
                                            sync_request_version);
                return Err(Error::ProcessInvalidChunk(error_message));
            }
        }

        // Valid responses should not have a highest ledger info less than target
        if let Some(highest_li) = highest_li {
            let target_version = target_li.ledger_info().version();
            let highest_version = highest_li.ledger_info().version();
            if target_version > highest_version {
                let error_message = format!("Progressive ledger info has target version > highest version. Target: {}, highest: {}.",
                                            target_version,
                                            highest_version);
                return Err(Error::ProcessInvalidChunk(error_message));
            }
        }

        Ok(())
    }

    fn verify_response_with_waypoint_li(
        &mut self,
        waypoint_li: &LedgerInfoWithSignatures,
        end_of_epoch_li: &Option<LedgerInfoWithSignatures>,
    ) -> Result<(), Error> {
        if self.is_initialized() || self.sync_request.is_some() {
            return Err(Error::ReceivedWrongChunkType(
                "Received a waypoint ledger info, but we're already initialized!".into(),
            ));
        }

        // Valid waypoint responses should not have an end_of_epoch_li version > waypoint_li
        if let Some(end_of_epoch_li) = end_of_epoch_li {
            let end_of_epoch_version = end_of_epoch_li.ledger_info().version();
            let waypoint_version = waypoint_li.ledger_info().version();
            if end_of_epoch_version > waypoint_version {
                let error_message = format!("Waypoint ledger info version is less than the end_of_epoch_li version. Waypoint: {}, end_of_epoch_li: {}.",
                                            waypoint_version,
                                            end_of_epoch_version);
                return Err(Error::ProcessInvalidChunk(error_message));
            }
        }

        Ok(())
    }

    /// Logs the highest seen ledger info version based on the current syncing mode.
    fn log_highest_seen_version(&self, new_highest_li: Option<LedgerInfoWithSignatures>) {
        let current_highest_version = if !self.is_initialized() {
            self.waypoint.version()
        } else if let Some(sync_request) = self.sync_request.as_ref() {
            sync_request.target.ledger_info().version()
        } else if let Some(new_highest_li) = new_highest_li.as_ref() {
            new_highest_li.ledger_info().version()
        } else if let Some(target_ledger_info) = self.target_ledger_info.as_ref() {
            target_ledger_info.ledger_info().version()
        } else {
            self.local_state.synced_version()
        };

        let highest_seen_version = counters::get_version(counters::VersionType::Highest);
        let highest_version = cmp::max(current_highest_version, highest_seen_version);
        counters::set_version(counters::VersionType::Highest, highest_version);
    }

    /// Calculates the next version and epoch to request (assuming the given transaction list
    /// and ledger info will be applied successfully). Note: if no ledger info is specified,
    /// we assume the next chunk will be for our current epoch.
    fn calculate_new_known_version_and_epoch(
        &mut self,
        txn_list_with_proof: TransactionListWithProof,
        ledger_info: Option<LedgerInfoWithSignatures>,
    ) -> Result<(u64, u64), Error> {
        let new_version = self
            .local_state
            .synced_version()
            .checked_add(txn_list_with_proof.len() as u64)
            .ok_or_else(|| {
                Error::IntegerOverflow("Potential state sync version has overflown".into())
            })?;

        let mut new_epoch = self.local_state.trusted_epoch();
        if let Some(ledger_info) = ledger_info {
            if ledger_info.ledger_info().version() == new_version
                && ledger_info.ledger_info().ends_epoch()
            {
                // This chunk is going to finish the current epoch. Choose the next one.
                new_epoch = new_epoch.checked_add(1).ok_or_else(|| {
                    Error::IntegerOverflow("Potential state sync epoch has overflown".into())
                })?;
            }
        }

        Ok((new_version, new_epoch))
    }

    /// Returns a chunk target for the highest available synchronization.
    fn create_highest_available_chunk_target(
        &self,
        target_ledger_info: Option<LedgerInfoWithSignatures>,
    ) -> TargetType {
        TargetType::HighestAvailable {
            target_li: target_ledger_info,
            timeout_ms: self.config.long_poll_timeout_ms,
        }
    }

    /// Returns a chunk target for consensus request synchronization.
    fn create_sync_request_chunk_target(&self, known_version: u64) -> Result<TargetType, Error> {
        if let Some(sync_request) = &self.sync_request {
            let target_version = sync_request.target.ledger_info().version();
            if target_version <= known_version {
                Err(Error::SyncedBeyondTarget(known_version, target_version))
            } else {
                let chunk_target =
                    self.create_highest_available_chunk_target(Some(sync_request.target.clone()));
                Ok(chunk_target)
            }
        } else {
            Err(Error::NoSyncRequestFound(
                "Unable to create a sync request chunk target".into(),
            ))
        }
    }

    /// Returns a chunk target for waypoint synchronization.
    fn create_waypoint_chunk_target(&self) -> TargetType {
        let waypoint_version = self.waypoint.version();
        TargetType::Waypoint(waypoint_version)
    }

    /// Processing chunk responses that carry a LedgerInfo that should be verified using the
    /// current local trusted validator set.
    fn process_response_with_target_and_highest(
        &mut self,
        txn_list_with_proof: TransactionListWithProof,
        response_li: LedgerInfoWithSignatures,
        new_highest_li: Option<LedgerInfoWithSignatures>,
    ) -> Result<(), Error> {
        // Optimistically calculate the new known version and epoch (assume the current chunk
        // is applied successfully).
        let (known_version, known_epoch) = self.calculate_new_known_version_and_epoch(
            txn_list_with_proof.clone(),
            Some(response_li.clone()),
        )?;

        // Send the next chunk request based on the sync mode (sync request or highest available).
        if self.sync_request.is_some() {
            match self.create_sync_request_chunk_target(known_version) {
                Ok(chunk_target) => {
                    // Send the chunk request and log any errors. If errors are logged
                    // continue processing the chunk.
                    let _ = self.send_chunk_request_and_log_error(
                        known_version,
                        known_epoch,
                        chunk_target,
                        LogEntry::ProcessChunkResponse,
                    );
                }
                Err(error) => {
                    error!(LogSchema::new(LogEntry::SendChunkRequest).error(&error));
                }
            }
        } else {
            let mut new_target_ledger_info = None;
            if let Some(target_ledger_info) = self.target_ledger_info.clone() {
                if known_version < target_ledger_info.ledger_info().version() {
                    new_target_ledger_info = Some(target_ledger_info);
                }
            }
            // Send the chunk request and log any errors. If errors are logged
            // continue processing the chunk.
            let _ = self.send_chunk_request_and_log_error(
                known_version,
                known_epoch,
                self.create_highest_available_chunk_target(new_target_ledger_info),
                LogEntry::ProcessChunkResponse,
            );
        }

        // Validate chunk ledger infos
        self.local_state.verify_ledger_info(&response_li)?;
        if let Some(new_highest_li) = new_highest_li.clone() {
            if new_highest_li != response_li {
                self.local_state.verify_ledger_info(&new_highest_li)?;
            }
        }

        // Validate and store the chunk
        self.log_highest_seen_version(new_highest_li.clone());
        self.validate_and_store_chunk(txn_list_with_proof, response_li, None)?;

        // Need to sync with local storage to update synced version
        self.sync_state_with_local_storage()?;
        let synced_version = self.local_state.synced_version();

        // Check if we've synced beyond our current target ledger info
        if let Some(target_ledger_info) = &self.target_ledger_info {
            if synced_version >= target_ledger_info.ledger_info().version() {
                self.target_ledger_info = None;
            }
        }

        // If we don't have a target ledger info, check if the new highest
        // is appropriate for us.
        if self.target_ledger_info.is_none() {
            if let Some(new_highest_li) = new_highest_li {
                if synced_version < new_highest_li.ledger_info().version() {
                    self.target_ledger_info = Some(new_highest_li);
                }
            }
        }

        Ok(())
    }

    /// Processing chunk responses that carry a LedgerInfo corresponding to the waypoint.
    fn process_response_with_waypoint_li(
        &mut self,
        txn_list_with_proof: TransactionListWithProof,
        waypoint_li: LedgerInfoWithSignatures,
        end_of_epoch_li: Option<LedgerInfoWithSignatures>,
    ) -> Result<(), Error> {
        // Optimistically calculate the new known version and epoch (assume the current chunk
        // is applied successfully).
        let (known_version, known_epoch) = self.calculate_new_known_version_and_epoch(
            txn_list_with_proof.clone(),
            end_of_epoch_li.clone(),
        )?;
        if known_version < self.waypoint.version() {
            // Send the chunk request and log any errors. If errors are logged
            // continue processing the chunk.
            let _ = self.send_chunk_request_and_log_error(
                known_version,
                known_epoch,
                self.create_waypoint_chunk_target(),
                LogEntry::ProcessChunkResponse,
            );
        }

        // Verify the end_of_epoch_li against local state and ensure the version
        // corresponds to the version at the end of the chunk.
        // The executor expects that when it is passed an end_of_epoch_li to commit,
        // it is going to execute/commit transactions leading up to that li, so we
        // also verify that the end_of_epoch_li actually ends the epoch.
        let end_of_epoch_li_to_commit = if let Some(end_of_epoch_li) = end_of_epoch_li {
            self.local_state.verify_ledger_info(&end_of_epoch_li)?;

            let ledger_info = end_of_epoch_li.ledger_info();
            if !ledger_info.ends_epoch() {
                return Err(Error::ProcessInvalidChunk(
                    "Received waypoint ledger info with an end_of_epoch_li that does not end the epoch!".into(),
                ));
            }

            // If we're now at the end of epoch version (i.e., known_version is the same as the
            // end_of_epoch_li version), the end_of_epoch_li should be passed to storage so that we
            // can commit the end_of_epoch_li. If not, storage should only sync the given chunk.
            if ledger_info.version() == known_version {
                Some(end_of_epoch_li)
            } else {
                None
            }
        } else {
            None
        };
        self.waypoint
            .verify(waypoint_li.ledger_info())
            .map_err(|error| {
                Error::UnexpectedError(format!("Waypoint verification failed: {}", error))
            })?;

        self.validate_and_store_chunk(txn_list_with_proof, waypoint_li, end_of_epoch_li_to_commit)?;
        self.log_highest_seen_version(None);

        Ok(())
    }

    // Assumes that the target LI has been already verified by the caller.
    fn validate_and_store_chunk(
        &mut self,
        txn_list_with_proof: TransactionListWithProof,
        target: LedgerInfoWithSignatures,
        intermediate_end_of_epoch_li: Option<LedgerInfoWithSignatures>,
    ) -> Result<(), Error> {
        let target_epoch = target.ledger_info().epoch();
        let target_version = target.ledger_info().version();
        let local_epoch = self.local_state.committed_epoch();
        let local_version = self.local_state.committed_version();
        if (target_epoch, target_version) <= (local_epoch, local_version) {
            warn!(
                LogSchema::event_log(LogEntry::ProcessChunkResponse, LogEvent::OldResponseLI)
                    .local_li_version(local_version)
                    .local_epoch(local_epoch),
                response_li_version = target_version,
                response_li_epoch = target_epoch
            );
            return Ok(());
        }

        self.executor_proxy
            .execute_chunk(txn_list_with_proof, target, intermediate_end_of_epoch_li)
    }

    /// Returns true if consensus is currently executing and state sync should
    /// therefore not write to storage. Reads are still permitted (e.g., to
    /// handle chunk requests).
    fn is_consensus_executing(&mut self) -> bool {
        self.is_initialized() && self.role == RoleType::Validator && self.sync_request.is_none()
    }

    /// Ensures that state sync is making progress:
    /// * Kick starts the initial sync process (e.g., syncing to a waypoint or target).
    /// * Issues a new request if too much time has passed since the last request was sent.
    fn check_progress(&mut self) -> Result<(), Error> {
        if self.is_consensus_executing() {
            return Ok(()); // No need to check progress or issue any requests (consensus is running).
        }

        // Check if the sync request has timed out (i.e., if we aren't committing fast enough)
        if let Some(sync_request) = self.sync_request.as_ref() {
            let timeout_between_commits =
                Duration::from_millis(self.config.sync_request_timeout_ms);
            let commit_deadline = sync_request
                .last_commit_timestamp
                .checked_add(timeout_between_commits)
                .ok_or_else(|| {
                    Error::IntegerOverflow("The commit deadline timestamp has overflown!".into())
                })?;

            // Check if the commit deadline has been exceeded.
            if SystemTime::now().duration_since(commit_deadline).is_ok() {
                counters::SYNC_REQUEST_RESULT
                    .with_label_values(&[counters::TIMEOUT_LABEL])
                    .inc();
                warn!(LogSchema::event_log(
                    LogEntry::SyncRequest,
                    LogEvent::Timeout
                ));

                // Remove the sync request and notify consensus that the request timed out!
                if let Some(sync_request) = self.sync_request.take() {
                    if let Err(e) = Self::send_sync_req_callback(
                        sync_request,
                        Err(Error::UnexpectedError("Sync request timed out!".into())),
                    ) {
                        error!(
                            LogSchema::event_log(LogEntry::SyncRequest, LogEvent::CallbackFail)
                                .error(&e)
                        );
                    }
                }
            }
        }

        // If the coordinator didn't make progress by the expected time or did not
        // send a request for the current local synced version, issue a new request.
        let known_version = self.local_state.synced_version();
        if self.request_manager.has_request_timed_out(known_version)? {
            counters::TIMEOUT.inc();
            warn!(LogSchema::new(LogEntry::Timeout).version(known_version));

            let trusted_epoch = self.local_state.trusted_epoch();
            let chunk_target = if !self.is_initialized() {
                self.create_waypoint_chunk_target()
            } else if self.sync_request.is_some() {
                self.create_sync_request_chunk_target(known_version)?
            } else {
                self.create_highest_available_chunk_target(self.target_ledger_info.clone())
            };
            self.send_chunk_request_and_log_error(
                known_version,
                trusted_epoch,
                chunk_target,
                LogEntry::Timeout,
            )
        } else {
            Ok(())
        }
    }

    /// Sends a chunk request with a given `known_version`, `known_epoch` and `chunk_target`.
    /// Immediately logs any errors returned by the operation using the given log entry.
    fn send_chunk_request_and_log_error(
        &mut self,
        known_version: u64,
        known_epoch: u64,
        chunk_target: TargetType,
        log_entry: LogEntry,
    ) -> Result<(), Error> {
        if let Err(error) =
            self.send_chunk_request_with_target(known_version, known_epoch, chunk_target)
        {
            error!(
                LogSchema::event_log(log_entry, LogEvent::SendChunkRequestFail)
                    .version(known_version)
                    .local_epoch(known_epoch)
                    .error(&error)
            );
            Err(error)
        } else {
            Ok(())
        }
    }

    /// Sends a chunk request with a given `known_version`, `known_epoch` and `target`.
    fn send_chunk_request_with_target(
        &mut self,
        known_version: u64,
        known_epoch: u64,
        target: TargetType,
    ) -> Result<(), Error> {
        if self.request_manager.no_available_peers() {
            warn!(LogSchema::event_log(
                LogEntry::SendChunkRequest,
                LogEvent::MissingPeers
            ));
            return Err(Error::NoAvailablePeers(
                "No peers to send chunk request to!".into(),
            ));
        }

        let target_version = target
            .version()
            .unwrap_or_else(|| known_version.wrapping_add(1));
        counters::set_version(counters::VersionType::Target, target_version);

        let req = GetChunkRequest::new(known_version, known_epoch, self.config.chunk_limit, target);
        self.request_manager.send_chunk_request(req)
    }

    fn deliver_subscription(
        &mut self,
        peer: PeerNetworkId,
        request_info: PendingRequestInfo,
        local_version: u64,
    ) -> Result<(), Error> {
        let (target_li, highest_li) = self.calculate_target_and_highest_li(
            request_info.request_epoch,
            request_info.target_li,
            local_version,
        )?;

        self.deliver_chunk(
            peer,
            request_info.known_version,
            ResponseLedgerInfo::ProgressiveLedgerInfo {
                target_li,
                highest_li,
            },
            request_info.chunk_limit,
        )
    }

    /// The function is called after the local storage is updated with new transactions:
    /// it might deliver chunks for the subscribers that have been waiting with the long polls.
    ///
    /// Note that it is possible to help the subscribers only with the transactions that match
    /// the highest ledger info in the local storage (some committed transactions are ahead of the
    /// latest ledger info and are not going to be used for helping the remote subscribers).
    /// The function assumes that the local state has been synced with storage.
    fn check_subscriptions(&mut self) {
        let highest_li_version = self.local_state.committed_version();

        let mut ready = vec![];
        self.subscriptions.retain(|peer, request_info| {
            // filter out expired peer requests
            if SystemTime::now()
                .duration_since(request_info.expiration_time)
                .is_ok()
            {
                return false;
            }
            if request_info.known_version < highest_li_version {
                ready.push((peer.clone(), request_info.clone()));
                false
            } else {
                true
            }
        });

        ready.into_iter().for_each(|(peer, request_info)| {
            let result_label = if let Err(err) =
                self.deliver_subscription(peer.clone(), request_info, highest_li_version)
            {
                error!(LogSchema::new(LogEntry::SubscriptionDeliveryFail)
                    .peer(&peer)
                    .error(&err));
                counters::FAIL_LABEL
            } else {
                counters::SUCCESS_LABEL
            };
            counters::SUBSCRIPTION_DELIVERY_COUNT
                .with_label_values(&[
                    &peer.raw_network_id().to_string(),
                    &peer.peer_id().to_string(),
                    result_label,
                ])
                .inc();
        });
    }

    fn send_sync_req_callback(sync_req: SyncRequest, msg: Result<(), Error>) -> Result<(), Error> {
        sync_req.callback.send(msg).map_err(|failed_msg| {
            counters::FAILED_CHANNEL_SEND
                .with_label_values(&[counters::CONSENSUS_SYNC_REQ_CALLBACK])
                .inc();
            Error::UnexpectedError(format!(
                "Consensus sync request callback error - failed to send the following message: {:?}",
                failed_msg
            ))
        })
    }

    fn send_initialization_callback(
        callback: oneshot::Sender<Result<(), Error>>,
    ) -> Result<(), Error> {
        match callback.send(Ok(())) {
            Err(error) => {
                counters::FAILED_CHANNEL_SEND
                    .with_label_values(&[counters::WAYPOINT_INIT_CALLBACK])
                    .inc();
                Err(Error::CallbackSendFailed(format!(
                    "Waypoint initialization callback error - failed to send following msg: {:?}",
                    error
                )))
            }
            _ => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        chunk_request::{GetChunkRequest, TargetType},
        chunk_response::{GetChunkResponse, ResponseLedgerInfo},
        client::SyncRequest,
        coordinator::StateSyncCoordinator,
        error::Error,
        executor_proxy::ExecutorProxy,
        network::StateSyncMessage,
        shared_components::{test_utils, test_utils::create_coordinator_with_config_and_waypoint},
    };
    use diem_config::{
        config::{NodeConfig, PeerNetworkId, PeerRole, RoleType},
        network_id::{NetworkId, NodeNetworkId},
    };
    use diem_crypto::{
        ed25519::{Ed25519PrivateKey, Ed25519Signature},
        HashValue, PrivateKey, Uniform,
    };
    use diem_mempool::CommitResponse;
    use diem_types::{
        account_address::AccountAddress,
        block_info::BlockInfo,
        chain_id::ChainId,
        ledger_info::{LedgerInfo, LedgerInfoWithSignatures},
        proof::TransactionListProof,
        transaction::{
            RawTransaction, Script, SignedTransaction, Transaction, TransactionListWithProof,
            TransactionPayload, Version,
        },
        waypoint::Waypoint,
        PeerId,
    };
    use futures::{channel::oneshot, executor::block_on};
    use mempool_notifications::MempoolNotifier;
    use netcore::transport::ConnectionOrigin;
    use network::transport::ConnectionMetadata;
    use std::{collections::BTreeMap, time::SystemTime};

    #[test]
    fn test_process_sync_request() {
        // Create a coordinator for a full node
        let mut full_node_coordinator = test_utils::create_full_node_coordinator();

        // Verify that fullnodes can't process sync requests
        let (sync_request, _) = create_sync_request_at_version(0);
        let process_result = full_node_coordinator.process_sync_request(sync_request);
        if !matches!(process_result, Err(Error::FullNodeSyncRequest)) {
            panic!(
                "Expected an full node sync request error, but got: {:?}",
                process_result
            );
        }

        // Create a coordinator for a validator node
        let mut validator_coordinator = test_utils::create_validator_coordinator();

        // Perform sync request for version that matches initial waypoint version
        let (sync_request, mut callback_receiver) = create_sync_request_at_version(0);
        validator_coordinator
            .process_sync_request(sync_request)
            .unwrap();
        match callback_receiver.try_recv() {
            Ok(Some(result)) => {
                assert!(result.is_ok())
            }
            result => panic!("Expected okay but got: {:?}", result),
        };

        // Create validator coordinator with waypoint higher than 0
        let waypoint_version = 10;
        let waypoint_ledger_info = create_ledger_info_at_version(waypoint_version);
        let waypoint = Waypoint::new_any(waypoint_ledger_info.ledger_info());
        let mut validator_coordinator =
            create_coordinator_with_config_and_waypoint(NodeConfig::default(), waypoint);

        // Verify coordinator won't process sync requests as it's not yet initialized
        let (sync_request, mut callback_receiver) = create_sync_request_at_version(10);
        let process_result = validator_coordinator.process_sync_request(sync_request);
        if !matches!(process_result, Err(Error::UninitializedError(..))) {
            panic!(
                "Expected an uninitialized error, but got: {:?}",
                process_result
            );
        }
        let callback_result = callback_receiver.try_recv();
        if !matches!(callback_result, Err(_)) {
            panic!("Expected error but got: {:?}", callback_result);
        }

        // TODO(joshlind): add a check for syncing to old versions once we support storage
        // modifications in unit tests.
    }

    #[test]
    fn test_get_sync_state() {
        // Create a coordinator for a validator node
        let mut validator_coordinator = test_utils::create_validator_coordinator();

        // Get the sync state from state sync
        let (callback_sender, mut callback_receiver) = oneshot::channel();
        validator_coordinator
            .get_sync_state(callback_sender)
            .unwrap();
        match callback_receiver.try_recv() {
            Ok(Some(sync_state)) => {
                assert_eq!(sync_state.committed_version(), 0);
            }
            result => panic!("Expected okay but got: {:?}", result),
        };

        // Drop the callback receiver and verify error
        let (callback_sender, _) = oneshot::channel();
        let sync_state_result = validator_coordinator.get_sync_state(callback_sender);
        if !matches!(sync_state_result, Err(Error::CallbackSendFailed(..))) {
            panic!("Expected error but got: {:?}", sync_state_result);
        }
    }

    #[test]
    fn test_wait_for_initialization() {
        // Create a coordinator for a validator node
        let mut validator_coordinator = test_utils::create_validator_coordinator();

        // Check already initialized returns immediately
        let (callback_sender, mut callback_receiver) = oneshot::channel();
        validator_coordinator
            .wait_for_initialization(callback_sender)
            .unwrap();
        match callback_receiver.try_recv() {
            Ok(Some(result)) => {
                assert!(result.is_ok())
            }
            result => panic!("Expected okay but got: {:?}", result),
        };

        // Drop the callback receiver and verify error
        let (callback_sender, _) = oneshot::channel();
        let initialization_result = validator_coordinator.wait_for_initialization(callback_sender);
        if !matches!(initialization_result, Err(Error::CallbackSendFailed(..))) {
            panic!("Expected error but got: {:?}", initialization_result);
        }

        // Create a coordinator with the waypoint version higher than 0
        let waypoint_version = 10;
        let waypoint_ledger_info = create_ledger_info_at_version(waypoint_version);
        let waypoint = Waypoint::new_any(waypoint_ledger_info.ledger_info());
        let mut validator_coordinator =
            create_coordinator_with_config_and_waypoint(NodeConfig::default(), waypoint);

        // Verify callback is not executed as state sync is not yet initialized
        let (callback_sender, mut callback_receiver) = oneshot::channel();
        validator_coordinator
            .wait_for_initialization(callback_sender)
            .unwrap();
        let callback_result = callback_receiver.try_recv();
        if !matches!(callback_result, Ok(None)) {
            panic!("Expected none but got: {:?}", callback_result);
        }

        // TODO(joshlind): add a check that verifies the callback is executed once we can
        // update storage in the unit tests.
    }

    #[test]
    fn test_process_commit_notification() {
        // Create a coordinator for a validator node
        let mut validator_coordinator = test_utils::create_validator_coordinator();

        // Verify that a commit notification with no transactions doesn't error!
        block_on(validator_coordinator.process_commit_notification(vec![], None, vec![], None))
            .unwrap();

        // Verify that consensus is sent a commit ack when everything works
        let (callback_sender, mut callback_receiver) =
            oneshot::channel::<Result<CommitResponse, Error>>();
        block_on(validator_coordinator.process_commit_notification(
            vec![],
            Some(callback_sender),
            vec![],
            None,
        ))
        .unwrap();
        let callback_result = callback_receiver.try_recv();
        if !matches!(callback_result, Ok(Some(Ok(..)))) {
            panic!("Expected an okay result but got: {:?}", callback_result);
        }

        // TODO(joshlind): verify that mempool is sent the correct transactions!
        let (callback_sender, _callback_receiver) =
            oneshot::channel::<Result<CommitResponse, Error>>();
        let committed_transactions = vec![create_test_transaction()];
        block_on(validator_coordinator.process_commit_notification(
            committed_transactions,
            Some(callback_sender),
            vec![],
            None,
        ))
        .unwrap();

        // TODO(joshlind): check initialized is fired when unit tests support storage
        // modifications.

        // TODO(joshlind): check sync request is called when unit tests support storage
        // modifications.

        // TODO(joshlind): test that long poll requests are handled appropriately when
        // new unit tests support this.

        // TODO(joshlind): test that reconfiguration events are handled appropriately
        // and listeners are notified.
    }

    #[test]
    fn test_check_progress() {
        // Create a coordinator for a validator node
        let mut validator_coordinator = test_utils::create_validator_coordinator();

        // Verify no error is returned when consensus is running
        validator_coordinator.check_progress().unwrap();

        // Send a sync request to state sync (to mark that consensus is no longer running)
        let (sync_request, _) = create_sync_request_at_version(1);
        let _ = validator_coordinator.process_sync_request(sync_request);

        // Verify the no available peers error is returned
        let progress_result = validator_coordinator.check_progress();
        if !matches!(progress_result, Err(Error::NoAvailablePeers(..))) {
            panic!("Expected an err result but got: {:?}", progress_result);
        }

        // Create validator coordinator with tiny state sync timeout
        let mut node_config = NodeConfig::default();
        node_config.base.role = RoleType::Validator;
        node_config.state_sync.sync_request_timeout_ms = 0;
        let mut validator_coordinator =
            create_coordinator_with_config_and_waypoint(node_config, Waypoint::default());

        // Set a new sync request
        let (sync_request, mut callback_receiver) = create_sync_request_at_version(1);
        let _ = validator_coordinator.process_sync_request(sync_request);

        // Verify sync request timeout notifies the callback
        validator_coordinator.check_progress().unwrap_err();
        let callback_result = callback_receiver.try_recv();
        if !matches!(callback_result, Ok(Some(Err(..)))) {
            panic!("Expected an err result but got: {:?}", callback_result);
        }

        // TODO(joshlind): check request resend after timeout.

        // TODO(joshlind): check overflow error returns.

        // TODO(joshlind): test that check progress passes when there are valid peers.
    }

    #[test]
    fn test_new_and_lost_peers() {
        // Create a coordinator for a validator node
        let mut validator_coordinator = test_utils::create_validator_coordinator();

        // Create a public peer
        let node_network_id = NodeNetworkId::new(NetworkId::Public, 0);
        let peer_id = PeerId::random();
        let connection_metadata = ConnectionMetadata::mock_with_role_and_origin(
            peer_id,
            PeerRole::Validator,
            ConnectionOrigin::Inbound,
        );

        // Verify error is returned when adding peer that is not a valid peer
        let new_peer_result =
            validator_coordinator.process_new_peer(node_network_id, connection_metadata.clone());
        if !matches!(new_peer_result, Err(Error::InvalidStateSyncPeer(..))) {
            panic!(
                "Expected an invalid peer error but got: {:?}",
                new_peer_result
            );
        }

        // Verify the same error is not returned when adding a validator node
        let node_network_id = NodeNetworkId::new(NetworkId::Validator, 0);
        let new_peer_result =
            validator_coordinator.process_new_peer(node_network_id.clone(), connection_metadata);
        if matches!(new_peer_result, Err(Error::InvalidStateSyncPeer(..))) {
            panic!(
                "Expected not to receive an invalid peer error but got: {:?}",
                new_peer_result
            );
        }

        // Verify no error is returned when removing the node
        validator_coordinator
            .process_lost_peer(node_network_id, peer_id)
            .unwrap();
    }

    #[test]
    fn test_invalid_chunk_request_messages() {
        // Create a coordinator for a validator node
        let mut validator_coordinator = test_utils::create_validator_coordinator();

        // Constants for the chunk requests
        let peer_network_id = PeerNetworkId::random();
        let current_epoch = 0;
        let chunk_limit = 250;
        let timeout_ms = 1000;

        // Create chunk requests with a known version higher than the target
        let known_version = 100;
        let target_version = 10;
        let chunk_requests = create_chunk_requests(
            known_version,
            current_epoch,
            chunk_limit,
            target_version,
            timeout_ms,
        );

        // Verify invalid request errors are thrown
        verify_all_chunk_requests_are_invalid(
            &mut validator_coordinator,
            &peer_network_id,
            &chunk_requests,
        );

        // Create chunk requests with a current epoch higher than the target epoch
        let known_version = 0;
        let current_epoch = 100;
        let chunk_requests = create_chunk_requests(
            known_version,
            current_epoch,
            chunk_limit,
            target_version,
            timeout_ms,
        );

        // Verify invalid request errors are thrown
        verify_all_chunk_requests_are_invalid(
            &mut validator_coordinator,
            &peer_network_id,
            &chunk_requests[1..2], // Ignore waypoint request
        );

        // Create chunk requests with a chunk limit size of 0 (which is a pointless request)
        let chunk_limit = 0;
        let chunk_requests = create_chunk_requests(
            known_version,
            current_epoch,
            chunk_limit,
            target_version,
            timeout_ms,
        );

        // Verify invalid request errors are thrown
        verify_all_chunk_requests_are_invalid(
            &mut validator_coordinator,
            &peer_network_id,
            &chunk_requests,
        );

        // Create chunk requests with a long poll timeout of 0 (which is a pointless request)
        let chunk_limit = 0;
        let chunk_requests = create_chunk_requests(
            known_version,
            current_epoch,
            chunk_limit,
            target_version,
            timeout_ms,
        );

        // Verify invalid request errors are thrown
        verify_all_chunk_requests_are_invalid(
            &mut validator_coordinator,
            &peer_network_id,
            &chunk_requests,
        );
    }

    #[test]
    fn test_process_chunk_response_messages() {
        // Create a coordinator for a validator node
        let mut validator_coordinator = test_utils::create_validator_coordinator();

        // Create a peer and empty chunk responses
        let peer_network_id = PeerNetworkId::random_validator();
        let empty_chunk_responses = create_empty_chunk_responses(10);

        // Verify a consensus error is returned when processing each chunk
        for chunk_response in &empty_chunk_responses {
            let result = block_on(validator_coordinator.process_chunk_message(
                peer_network_id.network_id(),
                peer_network_id.peer_id(),
                chunk_response.clone(),
            ));
            if !matches!(result, Err(Error::ConsensusIsExecuting)) {
                panic!("Expected consensus executing error, got: {:?}", result);
            }
        }

        // Make a sync request (to force consensus to yield)
        let (sync_request, _) = create_sync_request_at_version(10);
        let _ = validator_coordinator.process_sync_request(sync_request);

        // Verify we now get a downstream error (as the peer is downstream to us)
        for chunk_response in &empty_chunk_responses {
            let result = block_on(validator_coordinator.process_chunk_message(
                peer_network_id.network_id(),
                peer_network_id.peer_id(),
                chunk_response.clone(),
            ));
            if !matches!(result, Err(Error::ReceivedChunkFromDownstream(..))) {
                panic!("Expected a downstream error, but got: {:?}", result);
            }
        }

        // Add the peer to our known peers
        process_new_peer_event(&mut validator_coordinator, &peer_network_id);

        // Verify we now get an empty chunk error
        for chunk_response in &empty_chunk_responses {
            let result = block_on(validator_coordinator.process_chunk_message(
                peer_network_id.network_id(),
                peer_network_id.peer_id(),
                chunk_response.clone(),
            ));
            if !matches!(result, Err(Error::ReceivedEmptyChunk(..))) {
                panic!("Expected an empty chunk error, got: {:?}", result);
            }
        }

        // Send a non-empty chunk with a version mismatch and verify a mismatch error is returned
        let chunk_responses = create_non_empty_chunk_responses(10);
        for chunk_response in &chunk_responses {
            let result = block_on(validator_coordinator.process_chunk_message(
                peer_network_id.network_id(),
                peer_network_id.peer_id(),
                chunk_response.clone(),
            ));
            if !matches!(result, Err(Error::ReceivedNonSequentialChunk(..))) {
                panic!("Expected a non-sequential error, but got: {:?}", result);
            }
        }
    }

    #[test]
    fn test_process_chunk_response_highest() {
        // Create a coordinator for a full node
        let mut full_node_coordinator = test_utils::create_full_node_coordinator();

        // Create a peer for the node and add the peer as a known peer
        let peer_network_id = PeerNetworkId::random_validator();
        process_new_peer_event(&mut full_node_coordinator, &peer_network_id);

        // Verify wrong chunk type for non-highest messages
        let chunk_responses = create_non_empty_chunk_responses(1);
        verify_all_chunk_responses_are_the_wrong_type(
            &mut full_node_coordinator,
            &peer_network_id,
            &chunk_responses[0..1], // Ignore the target and highest chunk responses
        );

        // Verify highest known version must be greater than target version
        let response_ledger_info = ResponseLedgerInfo::ProgressiveLedgerInfo {
            target_li: create_ledger_info_at_version(100),
            highest_li: Some(create_ledger_info_at_version(10)),
        };
        let highest_response = create_chunk_response_message(
            response_ledger_info,
            create_dummy_transaction_list_with_proof(1),
        );
        verify_all_chunk_responses_are_invalid(
            &mut full_node_coordinator,
            &peer_network_id,
            &[highest_response],
        );

        // Verify invalid ledger infos are rejected
        let response_ledger_info = ResponseLedgerInfo::ProgressiveLedgerInfo {
            target_li: create_ledger_info_at_version(100),
            highest_li: None,
        };
        let highest_response = create_chunk_response_message(
            response_ledger_info,
            create_dummy_transaction_list_with_proof(1),
        );
        verify_all_chunk_responses_are_invalid(
            &mut full_node_coordinator,
            &peer_network_id,
            &[highest_response],
        );
    }

    #[test]
    fn test_process_chunk_response_target() {
        // Create a coordinator for a validator
        let mut validator_coordinator = test_utils::create_validator_coordinator();

        // Create a peer for the node and add the peer as a known peer
        let peer_network_id = PeerNetworkId::random_validator();
        process_new_peer_event(&mut validator_coordinator, &peer_network_id);

        // Make a sync request (to force consensus to yield)
        let (sync_request, _) = create_sync_request_at_version(10);
        let _ = validator_coordinator.process_sync_request(sync_request);

        // Verify wrong chunk type for waypoint message
        let chunk_responses = create_non_empty_chunk_responses(1);
        verify_all_chunk_responses_are_the_wrong_type(
            &mut validator_coordinator,
            &peer_network_id,
            &chunk_responses[0..1], // Ignore the target and highest chunk responses
        );

        // Verify ledger info version doesn't exceed sync request version
        let ledger_info = create_ledger_info_at_version(100);
        let response_ledger_info = ResponseLedgerInfo::VerifiableLedgerInfo(ledger_info);
        let target_response = create_chunk_response_message(
            response_ledger_info,
            create_dummy_transaction_list_with_proof(1),
        );
        verify_all_chunk_responses_are_invalid(
            &mut validator_coordinator,
            &peer_network_id,
            &[target_response],
        );

        // Verify invalid ledger infos are rejected
        let ledger_info = create_ledger_info_at_version(5);
        let response_ledger_info = ResponseLedgerInfo::VerifiableLedgerInfo(ledger_info);
        let target_response = create_chunk_response_message(
            response_ledger_info,
            create_dummy_transaction_list_with_proof(1),
        );
        verify_all_chunk_responses_are_invalid(
            &mut validator_coordinator,
            &peer_network_id,
            &[target_response],
        );
    }

    #[test]
    fn test_process_chunk_response_waypoint() {
        // Create a coordinator for a validator node with waypoint version of 10
        let waypoint_ledger_info = create_ledger_info_at_version(10);
        let waypoint = Waypoint::new_any(waypoint_ledger_info.ledger_info());
        let mut validator_coordinator =
            create_coordinator_with_config_and_waypoint(NodeConfig::default(), waypoint);

        // Create a peer for the node and add the peer as a known peer
        let peer_network_id = PeerNetworkId::random_validator();
        process_new_peer_event(&mut validator_coordinator, &peer_network_id);

        // Verify wrong chunk type for non-waypoint messages
        let chunk_responses = create_non_empty_chunk_responses(1);
        verify_all_chunk_responses_are_the_wrong_type(
            &mut validator_coordinator,
            &peer_network_id,
            &chunk_responses[1..=2], // Ignore the waypoint chunk response
        );

        // Verify end of epoch version is less than waypoint version
        let response_ledger_info = ResponseLedgerInfo::LedgerInfoForWaypoint {
            waypoint_li: create_ledger_info_at_version(10),
            end_of_epoch_li: Some(create_ledger_info_at_version(100)),
        };
        let waypoint_response = create_chunk_response_message(
            response_ledger_info,
            create_dummy_transaction_list_with_proof(1),
        );
        verify_all_chunk_responses_are_invalid(
            &mut validator_coordinator,
            &peer_network_id,
            &[waypoint_response],
        );

        // Verify that invalid waypoint ledger infos are rejected
        let response_ledger_info = ResponseLedgerInfo::LedgerInfoForWaypoint {
            waypoint_li: create_ledger_info_at_version(10),
            end_of_epoch_li: Some(create_ledger_info_at_version(10)),
        };
        let waypoint_response = create_chunk_response_message(
            response_ledger_info,
            create_dummy_transaction_list_with_proof(1),
        );
        verify_all_chunk_responses_are_invalid(
            &mut validator_coordinator,
            &peer_network_id,
            &[waypoint_response],
        );
    }

    fn create_test_transaction() -> Transaction {
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

    fn create_ledger_info_at_version(version: Version) -> LedgerInfoWithSignatures {
        let block_info =
            BlockInfo::new(0, 0, HashValue::zero(), HashValue::zero(), version, 0, None);
        let ledger_info = LedgerInfo::new(block_info, HashValue::random());
        LedgerInfoWithSignatures::new(ledger_info, BTreeMap::new())
    }

    fn create_sync_request_at_version(
        version: Version,
    ) -> (SyncRequest, oneshot::Receiver<Result<(), Error>>) {
        // Create ledger info with signatures at given version
        let ledger_info = create_ledger_info_at_version(version);

        // Create sync request with target version and callback
        let (callback_sender, callback_receiver) = oneshot::channel();
        let sync_request = SyncRequest {
            callback: callback_sender,
            target: ledger_info,
            last_commit_timestamp: SystemTime::now(),
        };

        (sync_request, callback_receiver)
    }

    /// Creates a set of chunk requests (one for each type of possible request).
    /// The returned request types are: [waypoint, target, highest].
    fn create_chunk_requests(
        known_version: Version,
        current_epoch: u64,
        chunk_limit: u64,
        target_version: u64,
        timeout_ms: u64,
    ) -> Vec<StateSyncMessage> {
        // Create a waypoint chunk request
        let target = TargetType::Waypoint(target_version);
        let waypoint_request =
            create_chunk_request_message(known_version, current_epoch, chunk_limit, target);

        // Create a highest chunk request
        let target_li = Some(create_ledger_info_at_version(target_version));
        let target = TargetType::HighestAvailable {
            target_li,
            timeout_ms,
        };
        let highest_request =
            create_chunk_request_message(known_version, current_epoch, chunk_limit, target);

        // Create a target chunk request
        let target_ledger_info = create_ledger_info_at_version(target_version);
        let target = TargetType::TargetLedgerInfo(target_ledger_info);
        let target_request =
            create_chunk_request_message(known_version, current_epoch, chunk_limit, target);

        vec![waypoint_request, target_request, highest_request]
    }

    fn create_chunk_request_message(
        known_version: Version,
        current_epoch: u64,
        chunk_limit: u64,
        target: TargetType,
    ) -> StateSyncMessage {
        let chunk_request = GetChunkRequest::new(known_version, current_epoch, chunk_limit, target);
        StateSyncMessage::GetChunkRequest(Box::new(chunk_request))
    }

    fn create_dummy_transaction_list_with_proof(version: Version) -> TransactionListWithProof {
        TransactionListWithProof::new(
            vec![create_test_transaction()],
            None,
            Some(version),
            TransactionListProof::new_empty(),
        )
    }

    fn create_chunk_response_message(
        response_ledger_info: ResponseLedgerInfo,
        transaction_list_with_proof: TransactionListWithProof,
    ) -> StateSyncMessage {
        let chunk_response =
            GetChunkResponse::new(response_ledger_info, transaction_list_with_proof);
        StateSyncMessage::GetChunkResponse(Box::new(chunk_response))
    }

    fn create_empty_chunk_responses(version: Version) -> Vec<StateSyncMessage> {
        create_chunk_responses(version, TransactionListWithProof::new_empty())
    }

    fn create_non_empty_chunk_responses(version: Version) -> Vec<StateSyncMessage> {
        let transaction_list_with_proof = create_dummy_transaction_list_with_proof(version);
        create_chunk_responses(version, transaction_list_with_proof)
    }

    /// Creates a set of chunk responses (one for each type of possible response).
    /// The returned response types are: [waypoint, target, highest].
    fn create_chunk_responses(
        version: Version,
        transaction_list_with_proof: TransactionListWithProof,
    ) -> Vec<StateSyncMessage> {
        let ledger_info_at_version = create_ledger_info_at_version(version);

        // Create a waypoint chunk response
        let response_ledger_info = ResponseLedgerInfo::LedgerInfoForWaypoint {
            waypoint_li: ledger_info_at_version.clone(),
            end_of_epoch_li: None,
        };
        let waypoint_response = create_chunk_response_message(
            response_ledger_info,
            transaction_list_with_proof.clone(),
        );

        // Create a highest chunk response
        let response_ledger_info = ResponseLedgerInfo::ProgressiveLedgerInfo {
            target_li: ledger_info_at_version.clone(),
            highest_li: None,
        };
        let highest_response = create_chunk_response_message(
            response_ledger_info,
            transaction_list_with_proof.clone(),
        );

        // Create a target chunk response
        let response_ledger_info = ResponseLedgerInfo::VerifiableLedgerInfo(ledger_info_at_version);
        let target_response =
            create_chunk_response_message(response_ledger_info, transaction_list_with_proof);

        vec![waypoint_response, target_response, highest_response]
    }

    fn verify_all_chunk_requests_are_invalid(
        coordinator: &mut StateSyncCoordinator<ExecutorProxy, MempoolNotifier>,
        peer_network_id: &PeerNetworkId,
        requests: &[StateSyncMessage],
    ) {
        for request in requests {
            let result = block_on(coordinator.process_chunk_message(
                peer_network_id.network_id(),
                peer_network_id.peer_id(),
                request.clone(),
            ));
            if !matches!(result, Err(Error::InvalidChunkRequest(..))) {
                panic!("Expected an invalid chunk request, but got: {:?}", result);
            }
        }
    }

    fn verify_all_chunk_responses_are_invalid(
        coordinator: &mut StateSyncCoordinator<ExecutorProxy, MempoolNotifier>,
        peer_network_id: &PeerNetworkId,
        responses: &[StateSyncMessage],
    ) {
        for response in responses {
            let result = block_on(coordinator.process_chunk_message(
                peer_network_id.network_id(),
                peer_network_id.peer_id(),
                response.clone(),
            ));
            if !matches!(result, Err(Error::ProcessInvalidChunk(..))) {
                panic!("Expected invalid chunk error, but got: {:?}", result);
            }
        }
    }

    fn verify_all_chunk_responses_are_the_wrong_type(
        coordinator: &mut StateSyncCoordinator<ExecutorProxy, MempoolNotifier>,
        peer_network_id: &PeerNetworkId,
        responses: &[StateSyncMessage],
    ) {
        for response in responses {
            let result = block_on(coordinator.process_chunk_message(
                peer_network_id.network_id(),
                peer_network_id.peer_id(),
                response.clone(),
            ));
            if !matches!(result, Err(Error::ReceivedWrongChunkType(..))) {
                panic!("Expected wrong type error, but got: {:?}", result);
            }
        }
    }

    fn process_new_peer_event(
        coordinator: &mut StateSyncCoordinator<ExecutorProxy, MempoolNotifier>,
        peer: &PeerNetworkId,
    ) {
        let connection_metadata = ConnectionMetadata::mock_with_role_and_origin(
            peer.peer_id(),
            PeerRole::Validator,
            ConnectionOrigin::Outbound,
        );
        let _ = coordinator.process_new_peer(peer.network_id(), connection_metadata);
    }
}
