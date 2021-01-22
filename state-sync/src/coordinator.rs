// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    chunk_request::{GetChunkRequest, TargetType},
    chunk_response::{GetChunkResponse, ResponseLedgerInfo},
    client::{CoordinatorMessage, SyncRequest},
    counters,
    error::{
        Error,
        Error::{CallbackSendFailed, OldSyncRequestVersion, UninitializedError},
    },
    executor_proxy::ExecutorProxyTrait,
    logging::{LogEntry, LogEvent, LogSchema},
    network::{StateSyncEvents, StateSyncMessage, StateSyncSender},
    request_manager::RequestManager,
    shared_components::{PendingLedgerInfos, SyncState},
};
use anyhow::{bail, ensure, format_err, Result};
use diem_config::{
    config::{NodeConfig, PeerNetworkId, RoleType, StateSyncConfig},
    network_id::NodeNetworkId,
};
use diem_logger::prelude::*;
use diem_mempool::{CommitResponse, CommittedTransaction};
use diem_types::{
    contract_event::ContractEvent,
    ledger_info::LedgerInfoWithSignatures,
    transaction::{Transaction, TransactionListWithProof, Version},
    waypoint::Waypoint,
};
use fail::fail_point;
use futures::{
    channel::{mpsc, oneshot},
    stream::select_all,
    StreamExt,
};
use network::protocols::network::Event;
use std::{
    collections::HashMap,
    time::{Duration, SystemTime},
};
use tokio::time::{interval, timeout};
use tokio_stream::wrappers::IntervalStream;

const MEMPOOL_COMMIT_ACK_TIMEOUT_SECS: u64 = 5;

#[derive(Clone, Debug, PartialEq, Eq)]
struct PendingRequestInfo {
    expiration_time: SystemTime,
    known_version: u64,
    request_epoch: u64,
    limit: u64,
}

/// Coordination of the state sync process is driven by StateSyncCoordinator. The `start()`
/// function runs an infinite event loop and triggers actions based on external and internal
/// (local) requests. The coordinator works in two modes (depending on the role):
/// * FullNode: infinite stream of ChunkRequests is sent to the predefined static peers
/// (the parent is going to reply with a ChunkResponse if its committed version becomes
/// higher within the timeout interval).
/// * Validator: the ChunkRequests are generated on demand for a specific target LedgerInfo to
/// synchronize to.
pub(crate) struct StateSyncCoordinator<T> {
    // used to process client requests
    client_events: mpsc::UnboundedReceiver<CoordinatorMessage>,
    // used to send messages (e.g. notifications about newly committed txns) to mempool
    state_sync_to_mempool_sender: mpsc::Sender<diem_mempool::CommitNotification>,
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
    // network senders - (k, v) = (network ID, network sender)
    network_senders: HashMap<NodeNetworkId, StateSyncSender>,
    // Actor for sending chunk requests
    // Manages to whom and how to send chunk requests
    request_manager: RequestManager,
    // Optional sync request to be called when the target sync is reached
    sync_request: Option<SyncRequest>,
    // Ledger infos in the future that have not been committed yet
    pending_ledger_infos: PendingLedgerInfos,
    // Option initialization listener to be called when the coordinator is caught up with
    // its waypoint.
    initialization_listener: Option<oneshot::Sender<Result<()>>>,
    // queue of incoming long polling requests
    // peer will be notified about new chunk of transactions if it's available before expiry time
    subscriptions: HashMap<PeerNetworkId, PendingRequestInfo>,
    executor_proxy: T,
}

impl<T: ExecutorProxyTrait> StateSyncCoordinator<T> {
    pub fn new(
        client_events: mpsc::UnboundedReceiver<CoordinatorMessage>,
        state_sync_to_mempool_sender: mpsc::Sender<diem_mempool::CommitNotification>,
        network_senders: HashMap<NodeNetworkId, StateSyncSender>,
        node_config: &NodeConfig,
        waypoint: Waypoint,
        executor_proxy: T,
        initial_state: SyncState,
    ) -> Result<Self> {
        info!(LogSchema::event_log(LogEntry::Waypoint, LogEvent::Initialize).waypoint(waypoint));

        // Create a new request manager.
        let role = node_config.base.role;
        let tick_interval_ms = node_config.state_sync.tick_interval_ms;
        let retry_timeout_val = match role {
            RoleType::FullNode => tick_interval_ms
                .checked_add(node_config.state_sync.long_poll_timeout_ms)
                .ok_or_else(|| format_err!("Fullnode retry timeout has overflown."))?,
            RoleType::Validator => tick_interval_ms
                .checked_mul(2)
                .ok_or_else(|| format_err!("Validator retry timeout has overflown!"))?,
        };
        let request_manager = RequestManager::new(
            node_config.upstream.clone(),
            Duration::from_millis(retry_timeout_val),
            Duration::from_millis(node_config.state_sync.multicast_timeout_ms),
            network_senders.clone(),
        );

        // Create new pending ledger infos.
        let pending_ledger_infos =
            PendingLedgerInfos::new(node_config.state_sync.max_pending_li_limit);

        Ok(Self {
            client_events,
            state_sync_to_mempool_sender,
            local_state: initial_state,
            pending_ledger_infos,
            config: node_config.state_sync.clone(),
            role,
            waypoint,
            request_manager,
            network_senders,
            subscriptions: HashMap::new(),
            sync_request: None,
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
                                error!(LogSchema::new(LogEntry::SyncRequest).error(&e.into()));
                                counters::SYNC_REQUEST_RESULT.with_label_values(&[counters::FAIL_LABEL]).inc();
                            }
                        }
                        CoordinatorMessage::CommitNotification(notification) => {
                            let _timer = counters::PROCESS_COORDINATOR_MSG_LATENCY
                                .with_label_values(&[counters::COMMIT_MSG_LABEL])
                                .start_timer();
                            if let Err(e) = self.process_commit_notification(notification.committed_transactions, Some(notification.callback), notification.reconfiguration_events, None).await {
                                counters::CONSENSUS_COMMIT_FAIL_COUNT.inc();
                                error!(LogSchema::event_log(LogEntry::ConsensusCommit, LogEvent::PostCommitFail).error(&e.into()));
                            }
                        }
                        CoordinatorMessage::GetSyncState(callback) => {
                            let _ = self.get_sync_state(callback);
                        }
                        CoordinatorMessage::WaitForInitialization(cb_sender) => {
                            if let Err(e) = self.wait_for_initialization(cb_sender) {
                                error!(LogSchema::new(LogEntry::Waypoint).error(&e.into()));
                            }
                        }
                    };
                },
                (network_id, event) = network_events.select_next_some() => {
                    match event {
                        Event::NewPeer(peer_id, origin) => {
                            let peer = PeerNetworkId(network_id, peer_id);
                            self.request_manager.enable_peer(peer, origin);
                            self.check_progress();
                        }
                        Event::LostPeer(peer_id, origin) => {
                            let peer = PeerNetworkId(network_id, peer_id);
                            self.request_manager.disable_peer(&peer, origin);
                        }
                        Event::Message(peer_id, message) => self.process_one_message(PeerNetworkId(network_id.clone(), peer_id), message).await,
                        unexpected_event => {
                            counters::NETWORK_ERROR_COUNT.inc();
                            warn!(LogSchema::new(LogEntry::NetworkError),
                            "received unexpected network event: {:?}", unexpected_event);
                        },
                    }
                },
                _ = interval.select_next_some() => {
                    self.check_progress();
                }
            }
        }
    }

    pub(crate) async fn process_one_message(&mut self, peer: PeerNetworkId, msg: StateSyncMessage) {
        match msg {
            StateSyncMessage::GetChunkRequest(request) => {
                let _timer = counters::PROCESS_MSG_LATENCY
                    .with_label_values(&[
                        &peer.raw_network_id().to_string(),
                        &peer.peer_id().to_string(),
                        counters::CHUNK_REQUEST_MSG_LABEL,
                    ])
                    .start_timer();
                let result_label =
                    if let Err(err) = self.process_chunk_request(peer.clone(), *request.clone()) {
                        error!(
                            LogSchema::event_log(LogEntry::ProcessChunkRequest, LogEvent::Fail)
                                .peer(&peer)
                                .error(&err)
                                .local_li_version(self.local_state.committed_version())
                                .chunk_request(*request)
                        );
                        counters::FAIL_LABEL
                    } else {
                        counters::SUCCESS_LABEL
                    };
                counters::PROCESS_CHUNK_REQUEST_COUNT
                    .with_label_values(&[
                        &peer.raw_network_id().to_string(),
                        &peer.peer_id().to_string(),
                        result_label,
                    ])
                    .inc();
            }
            StateSyncMessage::GetChunkResponse(response) => {
                let _timer = counters::PROCESS_MSG_LATENCY
                    .with_label_values(&[
                        &peer.raw_network_id().to_string(),
                        &peer.peer_id().to_string(),
                        counters::CHUNK_RESPONSE_MSG_LABEL,
                    ])
                    .start_timer();
                self.process_chunk_response(&peer, *response).await;
            }
        }
    }

    /// Sync up coordinator state with the local storage
    /// and updates the pending ledger info accordingly
    fn sync_state_with_local_storage(&mut self) -> Result<()> {
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

        self.pending_ledger_infos
            .update(&self.local_state, self.config.chunk_limit)
    }

    /// Verify that the local state's latest LI version (i.e. committed version) has reached the waypoint version.
    fn is_initialized(&self) -> bool {
        self.waypoint.version() <= self.local_state.committed_version()
    }

    fn wait_for_initialization(
        &mut self,
        cb_sender: oneshot::Sender<Result<()>>,
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
        fail_point!("state_sync::process_sync_request_message", |_| {
            Err(crate::error::Error::UnexpectedError(
                "Injected error in process_sync_request_message".into(),
            ))
        });

        let local_li_version = self.local_state.committed_version();
        let target_version = request.target.ledger_info().version();
        debug!(
            LogSchema::event_log(LogEntry::SyncRequest, LogEvent::Received)
                .target_version(target_version)
                .local_li_version(local_li_version)
        );

        self.sync_state_with_local_storage()?;
        if !self.is_initialized() {
            return Err(UninitializedError(
                "Unable to process sync request message!".into(),
            ));
        }

        if target_version == local_li_version {
            return Ok(Self::send_sync_req_callback(request, Ok(()))?);
        }
        if target_version < local_li_version {
            Self::send_sync_req_callback(request, Err(format_err!("Sync request to old version")))?;
            return Err(OldSyncRequestVersion(target_version, local_li_version));
        }

        self.sync_request = Some(request);
        Ok(self.send_chunk_request(
            self.local_state.synced_version(),
            self.local_state.trusted_epoch(),
        )?)
    }

    /// Notifies consensus of the given commit response.
    /// Note: if a callback is not specified, the response isn't sent anywhere.
    fn notify_consensus_of_commit_response(
        &self,
        commit_response_message: String,
        callback: Option<oneshot::Sender<Result<CommitResponse>>>,
    ) -> Result<(), Error> {
        let commit_response = CommitResponse {
            msg: commit_response_message,
        };

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
        commit_callback: Option<oneshot::Sender<Result<CommitResponse>>>,
        reconfiguration_events: Vec<ContractEvent>,
        chunk_sender: Option<&PeerNetworkId>,
    ) -> Result<(), Error> {
        // We choose to re-sync the state with the storage as it's the simplest approach:
        // in case the performance implications of re-syncing upon every commit are high,
        // it's possible to manage some of the highest known versions in memory.
        self.sync_state_with_local_storage()?;
        self.update_sync_state_metrics_and_logs()?;

        // Notify mempool of commit
        let commit_response_message = match self
            .notify_mempool_of_committed_transactions(committed_transactions)
            .await
        {
            Ok(()) => "".into(), // Empty message means success -- this is fixed in another PR!
            Err(error) => {
                error!(LogSchema::new(LogEntry::CommitFlow).error(&error.clone().into()));
                format!("{}", error)
            }
        };

        // Notify consensus of the commit response
        if let Err(error) =
            self.notify_consensus_of_commit_response(commit_response_message, commit_callback)
        {
            error!(LogSchema::new(LogEntry::CommitFlow).error(&error.into()),);
        }

        // Check long poll subscriptions, update peer requests and sync request timestamp
        self.check_subscriptions();
        let synced_version = self.local_state.synced_version();
        self.request_manager.remove_requests(synced_version);
        if let Some(peer) = chunk_sender {
            self.request_manager.process_success_response(peer);
        }
        if let Some(mut req) = self.sync_request.as_mut() {
            req.last_progress_tst = SystemTime::now();
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
                debug!(
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
        // Get all user transactions from committed transactions
        let user_transactions = committed_transactions
            .iter()
            .filter_map(|transaction| match transaction {
                Transaction::UserTransaction(signed_txn) => Some(CommittedTransaction {
                    sender: signed_txn.sender(),
                    sequence_number: signed_txn.sequence_number(),
                }),
                _ => None,
            })
            .collect();

        // Create commit notification of user transactions for mempool
        let (callback_sender, callback_receiver) = oneshot::channel();
        let req = diem_mempool::CommitNotification {
            transactions: user_transactions,
            block_timestamp_usecs: self
                .local_state
                .committed_ledger_info()
                .ledger_info()
                .timestamp_usecs(),
            callback: callback_sender,
        };

        // Notify mempool of committed transactions
        if let Err(error) = self.state_sync_to_mempool_sender.try_send(req) {
            counters::COMMIT_FLOW_FAIL
                .with_label_values(&[counters::TO_MEMPOOL_LABEL])
                .inc();
            Err(Error::CallbackSendFailed(format!(
                "Failed to notify mempool of committed transactions! Error: {:?}",
                error
            )))
        } else if let Err(error) = timeout(
            Duration::from_secs(MEMPOOL_COMMIT_ACK_TIMEOUT_SECS),
            callback_receiver,
        )
        .await
        {
            counters::COMMIT_FLOW_FAIL
                .with_label_values(&[counters::FROM_MEMPOOL_LABEL])
                .inc();
            Err(Error::CallbackSendFailed(format!(
                "Did not receive ACK for commit notification from mempool! Error: {:?}",
                error
            )))
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
            Err(error) => Err(CallbackSendFailed(format!(
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
    ) -> Result<()> {
        debug!(
            LogSchema::event_log(LogEntry::ProcessChunkRequest, LogEvent::Received)
                .peer(&peer)
                .chunk_request(request.clone())
                .local_li_version(self.local_state.committed_version())
        );
        fail_point!("state_sync::process_chunk_request", |_| {
            Err(anyhow::anyhow!("Injected error in process_chunk_request"))
        });
        self.sync_state_with_local_storage()?;

        match request.target.clone() {
            TargetType::TargetLedgerInfo(li) => self.process_request_target_li(peer, request, li),
            TargetType::HighestAvailable {
                target_li,
                timeout_ms,
            } => self.process_request_highest_available(peer, request, target_li, timeout_ms),
            TargetType::Waypoint(waypoint_version) => {
                self.process_request_waypoint(peer, request, waypoint_version)
            }
        }
    }

    /// Processing requests with a specified target LedgerInfo.
    /// Assumes that the local state is uptodate with storage.
    fn process_request_target_li(
        &mut self,
        peer: PeerNetworkId,
        request: GetChunkRequest,
        target_li: LedgerInfoWithSignatures,
    ) -> Result<()> {
        let limit = std::cmp::min(request.limit, self.config.max_chunk_limit);
        let response_li = self.choose_response_li(request.current_epoch, Some(target_li))?;
        // In case known_version is lower than the requested ledger info an empty response might be
        // sent.
        self.deliver_chunk(
            peer,
            request.known_version,
            ResponseLedgerInfo::VerifiableLedgerInfo(response_li),
            limit,
        )
    }

    /// Processing requests with no target LedgerInfo (highest available) and potentially long
    /// polling.
    /// Assumes that the local state is uptodate with storage.
    fn process_request_highest_available(
        &mut self,
        peer: PeerNetworkId,
        request: GetChunkRequest,
        target_li: Option<LedgerInfoWithSignatures>,
        timeout_ms: u64,
    ) -> Result<()> {
        let limit = std::cmp::min(request.limit, self.config.max_chunk_limit);
        let timeout = std::cmp::min(timeout_ms, self.config.max_timeout_ms);

        // If there is nothing a node can help with, and the request supports long polling,
        // add it to the subscriptions.
        let local_version = self.local_state.committed_version();
        if local_version <= request.known_version && timeout > 0 {
            let expiration_time = SystemTime::now().checked_add(Duration::from_millis(timeout));
            if let Some(time) = expiration_time {
                let request_info = PendingRequestInfo {
                    expiration_time: time,
                    known_version: request.known_version,
                    request_epoch: request.current_epoch,
                    limit,
                };
                self.subscriptions.insert(peer, request_info);
            }
            return Ok(());
        }

        // If the request's epoch is in the past, `target_li` will be set to the end-of-epoch LI for that epoch
        let target_li = self.choose_response_li(request.current_epoch, target_li)?;
        // Only populate highest_li field if it is different from target_li
        let highest_li = if target_li.ledger_info().version() < local_version
            && target_li.ledger_info().epoch() == self.local_state.trusted_epoch()
        {
            Some(self.local_state.committed_ledger_info())
        } else {
            None
        };

        self.deliver_chunk(
            peer,
            request.known_version,
            ResponseLedgerInfo::ProgressiveLedgerInfo {
                target_li,
                highest_li,
            },
            limit,
        )
    }

    fn process_request_waypoint(
        &mut self,
        peer: PeerNetworkId,
        request: GetChunkRequest,
        waypoint_version: Version,
    ) -> Result<()> {
        let mut limit = std::cmp::min(request.limit, self.config.max_chunk_limit);
        ensure!(
            self.local_state.committed_version() >= waypoint_version,
            "Local version {} < requested waypoint version {}.",
            self.local_state.committed_version(),
            waypoint_version
        );
        ensure!(
            request.known_version < waypoint_version,
            "Waypoint request version {} is not smaller than waypoint {}",
            request.known_version,
            waypoint_version
        );

        // Retrieve the waypoint LI.
        let waypoint_li = self
            .executor_proxy
            .get_epoch_ending_ledger_info(waypoint_version)?;

        // Txns are up to the end of request epoch with the proofs relative to the waypoint LI.
        let end_of_epoch_li = if waypoint_li.ledger_info().epoch() > request.current_epoch {
            let end_of_epoch_li = self
                .executor_proxy
                .get_epoch_change_ledger_info(request.current_epoch)?;
            ensure!(
                end_of_epoch_li.ledger_info().version() >= request.known_version,
                "waypoint request's current_epoch (epoch {}, version {}) < waypoint request's known_version {}",
                end_of_epoch_li.ledger_info().epoch(),
                end_of_epoch_li.ledger_info().version(),
                request.known_version,
            );
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
    ) -> Result<()> {
        let txns = self
            .executor_proxy
            .get_chunk(known_version, limit, response_li.version())?;
        let chunk_response = GetChunkResponse::new(response_li, txns);
        let log = LogSchema::event_log(LogEntry::ProcessChunkRequest, LogEvent::DeliverChunk)
            .chunk_response(chunk_response.clone())
            .peer(&peer);
        let msg = StateSyncMessage::GetChunkResponse(Box::new(chunk_response));

        let network_sender = self
            .network_senders
            .get_mut(&peer.network_id())
            .expect("missing network sender");
        let send_result = network_sender.send_to(peer.peer_id(), msg);
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
            error!(log.error(&e.into()));
            format_err!("Network error in sending chunk response to {}", peer)
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
    ) -> Result<LedgerInfoWithSignatures> {
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

    /// Applies (= executes and stores) chunk to storage if `response` is valid
    /// Chunk response checks performed:
    /// - does chunk contain no transactions?
    /// - does chunk of transactions matches the local state's version?
    /// - verify LIs in chunk response against local state
    /// - execute and commit chunk
    /// Returns error if above chunk response checks fail or chunk was not able to be stored to storage, else
    /// return Ok(()) if above checks all pass and chunk was stored to storage
    fn apply_chunk(&mut self, peer: &PeerNetworkId, response: GetChunkResponse) -> Result<()> {
        debug!(
            LogSchema::event_log(LogEntry::ProcessChunkResponse, LogEvent::Received)
                .chunk_response(response.clone())
                .peer(peer)
        );
        fail_point!("state_sync::apply_chunk", |_| {
            Err(anyhow::anyhow!("Injected error in apply_chunk"))
        });

        if !self.request_manager.is_known_upstream_peer(peer) {
            counters::RESPONSE_FROM_DOWNSTREAM_COUNT
                .with_label_values(&[
                    &peer.raw_network_id().to_string(),
                    &peer.peer_id().to_string(),
                ])
                .inc();
            bail!("received chunk response from downstream");
        }

        let txn_list_with_proof = response.txn_list_with_proof.clone();
        let known_version = self.local_state.synced_version();
        let chunk_start_version =
            txn_list_with_proof
                .first_transaction_version
                .ok_or_else(|| {
                    self.request_manager.process_empty_chunk(&peer);
                    format_err!("[state sync] Empty chunk from {:?}", peer)
                })?;

        let expected_version = known_version
            .checked_add(1)
            .ok_or_else(|| format_err!("Expected version has overflown!"))?;
        if chunk_start_version != expected_version {
            // Old / wrong chunk.
            self.request_manager.process_chunk_version_mismatch(
                peer,
                chunk_start_version,
                known_version,
            )?;
        }

        let chunk_size = txn_list_with_proof.len() as u64;
        match response.response_li {
            ResponseLedgerInfo::VerifiableLedgerInfo(li) => {
                self.process_response_with_verifiable_li(txn_list_with_proof, li, None)
            }
            ResponseLedgerInfo::ProgressiveLedgerInfo {
                target_li,
                highest_li,
            } => {
                let highest_li = highest_li.unwrap_or_else(|| target_li.clone());
                ensure!(
                    target_li.ledger_info().version() <= highest_li.ledger_info().version(),
                    "Progressive ledger info received target LI {} higher than highest LI {}",
                    target_li,
                    highest_li
                );
                self.process_response_with_verifiable_li(
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
        .map_err(|e| {
            self.request_manager.process_invalid_chunk(&peer);
            format_err!("[state sync] failed to apply chunk: {}", e)
        })?;

        counters::STATE_SYNC_CHUNK_SIZE
            .with_label_values(&[
                &peer.raw_network_id().to_string(),
                &peer.peer_id().to_string(),
            ])
            .observe(chunk_size as f64);
        let new_version = known_version
            .checked_add(chunk_size)
            .ok_or_else(|| format_err!("New version has overflown!"))?;
        debug!(
            LogSchema::event_log(LogEntry::ProcessChunkResponse, LogEvent::ApplyChunkSuccess),
            "Applied chunk of size {}. Previous version: {}, new version {}",
            chunk_size,
            known_version,
            new_version
        );

        // The overall chunk processing duration is calculated starting from the very first attempt
        // until the commit
        if let Some(first_attempt_tst) = self.request_manager.get_first_request_time(known_version)
        {
            if let Ok(duration) = SystemTime::now().duration_since(first_attempt_tst) {
                counters::SYNC_PROGRESS_DURATION.observe_duration(duration);
            }
        }
        Ok(())
    }

    /// * Verifies and stores chunk in response
    /// * Triggers post-commit actions based on new local state after successful chunk processing in above step
    async fn process_chunk_response(&mut self, peer: &PeerNetworkId, response: GetChunkResponse) {
        // Part 0: ensure consensus isn't running, otherwise we might get a race with storage writes.
        if self.is_consensus_executing() {
            error!(LogSchema::event_log(
                LogEntry::ProcessChunkResponse,
                LogEvent::ConsensusIsRunning
            )
            .peer(peer)
            .error(&format_err!(
                "Received a chunk response but consensus is now running so we cannot process it!"
            )));
            return;
        }

        let new_txns = response.txn_list_with_proof.transactions.clone();
        // Part 1: check response, validate and store chunk
        // any errors thrown here should be for detecting actual bad chunks
        if let Err(e) = self.apply_chunk(peer, response) {
            // count, log, and exit
            error!(
                LogSchema::event_log(LogEntry::ProcessChunkResponse, LogEvent::ApplyChunkFail)
                    .peer(peer)
                    .error(&e)
            );
            counters::APPLY_CHUNK_COUNT
                .with_label_values(&[
                    &peer.raw_network_id().to_string(),
                    &peer.peer_id().to_string(),
                    counters::FAIL_LABEL,
                ])
                .inc();
            return;
        }

        counters::APPLY_CHUNK_COUNT
            .with_label_values(&[
                &peer.raw_network_id().to_string(),
                &peer.peer_id().to_string(),
                counters::SUCCESS_LABEL,
            ])
            .inc();

        // Part 2: post-chunk-process stage: process commit
        if let Err(error) = self
            .process_commit_notification(new_txns, None, vec![], Some(peer))
            .await
        {
            error!(
                LogSchema::event_log(LogEntry::ProcessChunkResponse, LogEvent::PostCommitFail)
                    .error(&error.into())
            );
        }
    }

    /// Processing chunk responses that carry a LedgerInfo that should be verified using the
    /// current local trusted validator set.
    fn process_response_with_verifiable_li(
        &mut self,
        txn_list_with_proof: TransactionListWithProof,
        response_li: LedgerInfoWithSignatures,
        // LI to verify and add to pending_ledger_infos
        // may be the same as response_li
        pending_li: Option<LedgerInfoWithSignatures>,
    ) -> Result<()> {
        ensure!(
            self.is_initialized(),
            "Response with a non-waypoint LI while still not initialized"
        );
        if let Some(sync_req) = self.sync_request.as_ref() {
            // Valid responses should not exceed the LI version of the request.
            if sync_req.target.ledger_info().version() < response_li.ledger_info().version() {
                bail!(
                    "[state sync] Response has an LI version {} higher than requested version {}.",
                    response_li.ledger_info().version(),
                    sync_req.target.ledger_info().version(),
                );
            }
        }
        // Optimistically fetch the next chunk assuming the current chunk is going to be applied
        // successfully.
        let new_version = self
            .local_state
            .synced_version()
            .checked_add(txn_list_with_proof.len() as u64)
            .ok_or_else(|| format_err!("New version has overflown!"))?;
        let new_epoch = if response_li.ledger_info().version() == new_version
            && response_li.ledger_info().ends_epoch()
        {
            // This chunk is going to finish the current epoch, optimistically request a chunk
            // from the next epoch.
            self.local_state
                .trusted_epoch()
                .checked_add(1)
                .ok_or_else(|| format_err!("New epoch has overflown!"))?
        } else {
            // Remain in the current epoch
            self.local_state.trusted_epoch()
        };
        self.local_state.verify_ledger_info(&response_li)?;
        if let Some(li) = pending_li {
            if li != response_li {
                self.local_state.verify_ledger_info(&li)?;
            }
            self.pending_ledger_infos.add_li(li);
        }
        self.validate_and_store_chunk(txn_list_with_proof, response_li, None)?;

        // need to sync with local storage to see whether response LI was actually committed
        // and update pending_ledger_infos accordingly
        self.sync_state_with_local_storage()?;
        let new_version = self.local_state.synced_version();

        // don't throw error for failed chunk request send, as this failure is not related to
        // validity of the chunk response itself
        if let Err(e) = self.send_chunk_request(new_version, new_epoch) {
            error!(LogSchema::event_log(
                LogEntry::ProcessChunkResponse,
                LogEvent::SendChunkRequestFail
            )
            .error(&e));
        }

        Ok(())
    }

    /// Processing chunk responses that carry a LedgerInfo corresponding to the waypoint.
    fn process_response_with_waypoint_li(
        &mut self,
        txn_list_with_proof: TransactionListWithProof,
        waypoint_li: LedgerInfoWithSignatures,
        end_of_epoch_li: Option<LedgerInfoWithSignatures>,
    ) -> Result<()> {
        ensure!(
            !self.is_initialized(),
            "Response with a waypoint LI but we're already initialized"
        );
        // Optimistically fetch the next chunk.
        let new_version = self
            .local_state
            .synced_version()
            .checked_add(txn_list_with_proof.len() as u64)
            .ok_or_else(|| format_err!("New version has overflown!"))?;
        // The epoch in the optimistic request (= new_epoch) should be the next epoch if the current chunk
        // is the last one in its epoch.
        let next_epoch = self
            .local_state
            .trusted_epoch()
            .checked_add(1)
            .ok_or_else(|| format_err!("Next epoch has overflown!"))?;
        let new_epoch = end_of_epoch_li
            .as_ref()
            .map_or(self.local_state.trusted_epoch(), |li| {
                if li.ledger_info().version() == new_version && li.ledger_info().ends_epoch() {
                    next_epoch
                } else {
                    self.local_state.trusted_epoch()
                }
            });
        if new_version < self.waypoint.version() {
            if let Err(e) = self.send_chunk_request(new_version, new_epoch) {
                error!(LogSchema::event_log(
                    LogEntry::ProcessChunkResponse,
                    LogEvent::SendChunkRequestFail
                )
                .error(&e));
            }
        }

        // verify the end-of-epoch LI for the following before passing it to execution:
        // * verify end-of-epoch-li against local state
        // * verify end-of-epoch-li's version corresponds to end-of-chunk version before
        // passing it to execution
        // * Executor expects that when it is passed an end-of-epoch LI, it is going to execute/commit
        // transactions leading up to that end-of-epoch LI
        // * verify end-of-epoch-li actually ends an epoch
        let end_of_epoch_li = end_of_epoch_li
            .map(|li| self.local_state.verify_ledger_info(&li).map(|_| li))
            .transpose()?
            .filter(|li| {
                li.ledger_info().version() == new_version && li.ledger_info().ends_epoch()
            });
        self.waypoint.verify(waypoint_li.ledger_info())?;
        self.validate_and_store_chunk(txn_list_with_proof, waypoint_li, end_of_epoch_li)
    }

    // Assumes that the target LI has been already verified by the caller.
    fn validate_and_store_chunk(
        &mut self,
        txn_list_with_proof: TransactionListWithProof,
        target: LedgerInfoWithSignatures,
        intermediate_end_of_epoch_li: Option<LedgerInfoWithSignatures>,
    ) -> Result<()> {
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
    /// * kick-starts initial sync process (= initialization syncing to waypoint)
    /// * issue a new request if too much time passed since requesting highest_synced_version + 1.
    fn check_progress(&mut self) {
        if self.request_manager.no_available_peers() {
            return;
        }

        if self.is_consensus_executing() {
            return; // No need to send out chunk requests: consensus is executing.
        }

        // check that we made progress in fulfilling consensus sync request
        let sync_request_expired = self.sync_request.as_ref().map_or(false, |req| {
            let default_timeout = Duration::from_millis(self.config.sync_request_timeout_ms);
            if let Some(tst) = req.last_progress_tst.checked_add(default_timeout) {
                return SystemTime::now().duration_since(tst).is_ok();
            }
            false
        });
        // notify consensus if sync request timed out
        if sync_request_expired {
            counters::SYNC_REQUEST_RESULT
                .with_label_values(&[counters::TIMEOUT_LABEL])
                .inc();
            warn!(LogSchema::event_log(
                LogEntry::SyncRequest,
                LogEvent::Timeout
            ));

            if let Some(sync_request) = self.sync_request.take() {
                if let Err(e) = Self::send_sync_req_callback(
                    sync_request,
                    Err(format_err!("request timed out")),
                ) {
                    error!(
                        LogSchema::event_log(LogEntry::SyncRequest, LogEvent::CallbackFail)
                            .error(&e)
                    );
                }
            }
        }

        let known_version = self.local_state.synced_version();

        // if coordinator didn't make progress by expected time or did not send a request for current
        // local synced version, issue new request
        if self
            .request_manager
            .check_timeout(known_version)
            .unwrap_or(false)
        {
            // log and count timeout
            counters::TIMEOUT.inc();
            warn!(LogSchema::new(LogEntry::Timeout).version(known_version));
            if let Err(e) = self.send_chunk_request(known_version, self.local_state.trusted_epoch())
            {
                error!(
                    LogSchema::event_log(LogEntry::Timeout, LogEvent::SendChunkRequestFail)
                        .version(known_version)
                        .error(&e)
                );
            }
        }
    }

    /// Sends a chunk request with a given `known_version` and `known_epoch`
    /// (might be chosen optimistically).
    fn send_chunk_request(&mut self, known_version: u64, known_epoch: u64) -> Result<()> {
        if self.request_manager.no_available_peers() {
            warn!(LogSchema::event_log(
                LogEntry::SendChunkRequest,
                LogEvent::MissingPeers
            ));
            bail!("No peers to send chunk request to");
        }

        let target = if !self.is_initialized() {
            let waypoint_version = self.waypoint.version();
            TargetType::Waypoint(waypoint_version)
        } else {
            match self.sync_request.as_ref() {
                None => {
                    TargetType::HighestAvailable {
                        // here, we need to ensure pending_ledger_infos is up-to-date with storage
                        // this is the responsibility of the caller of send_chunk_request
                        target_li: self.pending_ledger_infos.target_li(),
                        timeout_ms: self.config.long_poll_timeout_ms,
                    }
                }
                Some(sync_req) => {
                    let sync_target_version = sync_req.target.ledger_info().version();
                    if sync_target_version <= known_version {
                        // sync request is already fulfilled, so don't send chunk requests with this request as target
                        debug!(LogSchema::event_log(
                            LogEntry::SendChunkRequest,
                            LogEvent::OldSyncRequest
                        )
                        .target_version(sync_target_version)
                        .local_synced_version(known_version), "Sync request is already fulfilled, so no need to send chunk requests for this sync request");
                        return Ok(());
                    }
                    TargetType::TargetLedgerInfo(sync_req.target.clone())
                }
            }
        };

        let target_version = target
            .version()
            .unwrap_or_else(|| known_version.wrapping_add(1));
        counters::set_version(counters::VersionType::Target, target_version);
        let highest_version = self
            .pending_ledger_infos
            .highest_version()
            .unwrap_or(target_version);
        counters::set_version(counters::VersionType::Highest, highest_version);
        let req = GetChunkRequest::new(known_version, known_epoch, self.config.chunk_limit, target);
        self.request_manager.send_chunk_request(req)
    }

    fn deliver_subscription(
        &mut self,
        peer: PeerNetworkId,
        request_info: PendingRequestInfo,
    ) -> Result<()> {
        let response_li = self.choose_response_li(request_info.request_epoch, None)?;
        self.deliver_chunk(
            peer,
            request_info.known_version,
            ResponseLedgerInfo::VerifiableLedgerInfo(response_li),
            request_info.limit,
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
            let result_label =
                if let Err(err) = self.deliver_subscription(peer.clone(), request_info) {
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

    fn send_sync_req_callback(sync_req: SyncRequest, msg: Result<()>) -> Result<()> {
        sync_req.callback.send(msg).map_err(|failed_msg| {
            counters::FAILED_CHANNEL_SEND
                .with_label_values(&[counters::CONSENSUS_SYNC_REQ_CALLBACK])
                .inc();
            format_err!(
                "Consensus sync request callback error - failed to send following msg: {:?}",
                failed_msg
            )
        })
    }

    fn send_initialization_callback(callback: oneshot::Sender<Result<()>>) -> Result<(), Error> {
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
    use crate::{client::SyncRequest, error::Error, shared_components::test_utils};
    use anyhow::Result;
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
        transaction::{
            RawTransaction, Script, SignedTransaction, Transaction, TransactionPayload, Version,
        },
        waypoint::Waypoint,
    };
    use futures::{channel::oneshot, executor::block_on};
    use std::{collections::BTreeMap, time::SystemTime};

    #[test]
    fn test_process_sync_request() {
        let mut coordinator = test_utils::create_state_sync_coordinator_for_tests();

        // Perform sync request for version that matches initial waypoint version
        let (sync_request, mut callback_receiver) = create_sync_request_at_version(0);
        coordinator.process_sync_request(sync_request).unwrap();
        match callback_receiver.try_recv() {
            Ok(Some(result)) => {
                assert!(result.is_ok())
            }
            result => panic!("Expected okay but got: {:?}", result),
        };

        // Set coordinator waypoint to version higher than storage
        let waypoint_version = 10;
        let waypoint_ledger_info = create_ledger_info_at_version(waypoint_version);
        coordinator.waypoint = Waypoint::new_any(&waypoint_ledger_info.ledger_info());

        // Verify coordinator won't process sync requests as it's not yet initialized
        let (sync_request, mut callback_receiver) = create_sync_request_at_version(10);
        let process_result = coordinator.process_sync_request(sync_request);
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
        let mut coordinator = test_utils::create_state_sync_coordinator_for_tests();

        // Get the sync state from state sync
        let (callback_sender, mut callback_receiver) = oneshot::channel();
        coordinator.get_sync_state(callback_sender).unwrap();
        match callback_receiver.try_recv() {
            Ok(Some(sync_state)) => {
                assert_eq!(sync_state.committed_version(), 0);
            }
            result => panic!("Expected okay but got: {:?}", result),
        };

        // Drop the callback receiver and verify error
        let (callback_sender, _) = oneshot::channel();
        let sync_state_result = coordinator.get_sync_state(callback_sender);
        if !matches!(sync_state_result, Err(Error::CallbackSendFailed(..))) {
            panic!("Expected error but got: {:?}", sync_state_result);
        }
    }

    #[test]
    fn test_wait_for_initialization() {
        let mut coordinator = test_utils::create_state_sync_coordinator_for_tests();

        // Check already initialized returns immediately
        let (callback_sender, mut callback_receiver) = oneshot::channel();
        coordinator
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
        let initialization_result = coordinator.wait_for_initialization(callback_sender);
        if !matches!(initialization_result, Err(Error::CallbackSendFailed(..))) {
            panic!("Expected error but got: {:?}", initialization_result);
        }

        // Set the waypoint version higher than storage
        let waypoint_version = 10;
        let waypoint_ledger_info = create_ledger_info_at_version(waypoint_version);
        coordinator.waypoint = Waypoint::new_any(&waypoint_ledger_info.ledger_info());

        // Verify callback is not executed as state sync is not yet initialized
        let (callback_sender, mut callback_receiver) = oneshot::channel();
        coordinator
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
        let mut coordinator = test_utils::create_state_sync_coordinator_for_tests();

        // Verify that a commit notification with no transactions doesn't error!
        block_on(coordinator.process_commit_notification(vec![], None, vec![], None)).unwrap();

        // Verify that consensus is sent a commit ack when everything works
        let (callback_sender, mut callback_receiver) = oneshot::channel::<Result<CommitResponse>>();
        block_on(coordinator.process_commit_notification(
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
        let (callback_sender, _callback_receiver) = oneshot::channel::<Result<CommitResponse>>();
        let committed_transactions = vec![create_test_transaction()];
        block_on(coordinator.process_commit_notification(
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
    ) -> (SyncRequest, oneshot::Receiver<Result<()>>) {
        // Create ledger info with signatures at given version
        let ledger_info = create_ledger_info_at_version(version);

        // Create sync request with target version and callback
        let (callback_sender, callback_receiver) = oneshot::channel();
        let sync_request = SyncRequest {
            callback: callback_sender,
            target: ledger_info,
            last_progress_tst: SystemTime::now(),
        };

        (sync_request, callback_receiver)
    }
}
