// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    chunk_request::{GetChunkRequest, TargetType},
    chunk_response::{GetChunkResponse, ResponseLedgerInfo},
    counters,
    executor_proxy::ExecutorProxyTrait,
    network::{StateSynchronizerEvents, StateSynchronizerMsg, StateSynchronizerSender},
    peer_manager::{PeerManager, PeerScoreUpdateType},
    PeerId, SynchronizerState,
};
use anyhow::{bail, ensure, format_err, Result};
use futures::{
    channel::{mpsc, oneshot},
    stream::select_all,
    StreamExt,
};
use libra_config::config::{RoleType, StateSyncConfig};
use libra_logger::prelude::*;
use libra_mempool::{CommitNotification, CommitResponse, CommittedTransaction};
use libra_types::{
    contract_event::ContractEvent,
    ledger_info::LedgerInfoWithSignatures,
    transaction::{Transaction, TransactionListWithProof, Version},
    validator_change::{ValidatorChangeProof, VerifierType},
    waypoint::Waypoint,
};
use network::protocols::network::Event;
use std::{
    collections::HashMap,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::time::{interval, timeout};

pub(crate) struct SyncRequest {
    // The Result value returned to the caller is Error in case the StateSynchronizer failed to
    // reach the target (the LI in the storage remains unchanged as if nothing happened).
    pub callback: oneshot::Sender<Result<()>>,
    pub target: LedgerInfoWithSignatures,
}

pub(crate) struct EpochRetrievalRequest {
    pub start_epoch: u64,
    pub end_epoch: u64,
    pub callback: oneshot::Sender<Result<ValidatorChangeProof>>,
}

/// message used by StateSyncClient for communication with Coordinator
pub(crate) enum CoordinatorMessage {
    // used to initiate new sync
    Request(SyncRequest),
    // used to notify about new txn commit
    Commit(
        // committed transactions
        Vec<Transaction>,
        // reconfiguration events
        Vec<ContractEvent>,
        // callback for recipient to send response back to this sender
        oneshot::Sender<Result<CommitResponse>>,
    ),
    GetState(oneshot::Sender<SynchronizerState>),
    // used to generate epoch proof
    GetEpochProof(EpochRetrievalRequest),
    // Receive a notification via a given channel when coordinator is initialized.
    WaitInitialize(oneshot::Sender<Result<()>>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PendingRequestInfo {
    expiration_time: SystemTime,
    known_version: u64,
    request_epoch: u64,
    limit: u64,
}

/// Coordination of synchronization process is driven by SyncCoordinator, which `start()` function
/// runs an infinite event loop and triggers actions based on external / internal requests.
/// The coordinator can work in two modes:
/// * FullNode: infinite stream of ChunkRequests is sent to the predefined static peers
/// (the parent is going to reply with a ChunkResponse if its committed version becomes
/// higher within the timeout interval).
/// * Validator: the ChunkRequests are generated on demand for a specific target LedgerInfo to
/// synchronize to.
pub(crate) struct SyncCoordinator<T> {
    // used to process client requests
    client_events: mpsc::UnboundedReceiver<CoordinatorMessage>,
    // used to send messages (e.g. notifications about newly committed txns) to mempool
    state_sync_to_mempool_sender: mpsc::Sender<CommitNotification>,
    // Current state of the storage, which includes both the latest committed transaction and the
    // latest transaction covered by the LedgerInfo (see `SynchronizerState` documentation).
    // The state is updated via syncing with the local storage.
    local_state: SynchronizerState,
    // duration with the same version before the next attempt to get the next chunk
    retry_timeout: Duration,
    // config
    config: StateSyncConfig,
    // role of node
    role: RoleType,
    // An initial waypoint: for as long as the local version is less than a version determined by
    // waypoint a node is not going to be abl
    waypoint: Option<Waypoint>,
    // peers used for synchronization
    peer_manager: PeerManager,
    // Optional sync request to be called when the target sync is reached
    sync_request: Option<SyncRequest>,
    // Option initialization listener to be called when the coordinator is caught up with
    // its waypoint.
    initialization_listener: Option<oneshot::Sender<Result<()>>>,
    // queue of incoming long polling requests
    // peer will be notified about new chunk of transactions if it's available before expiry time
    subscriptions: HashMap<PeerId, PendingRequestInfo>,
    executor_proxy: T,
}

impl<T: ExecutorProxyTrait> SyncCoordinator<T> {
    pub fn new(
        client_events: mpsc::UnboundedReceiver<CoordinatorMessage>,
        state_sync_to_mempool_sender: mpsc::Sender<CommitNotification>,
        role: RoleType,
        waypoint: Option<Waypoint>,
        config: StateSyncConfig,
        executor_proxy: T,
        initial_state: SynchronizerState,
    ) -> Self {
        let upstream_peers = config.upstream_peers.upstream_peers.clone();
        let retry_timeout_val = match role {
            RoleType::FullNode => config.tick_interval_ms + config.long_poll_timeout_ms,
            RoleType::Validator => 2 * config.tick_interval_ms,
        };

        Self {
            client_events,
            state_sync_to_mempool_sender,
            local_state: initial_state,
            retry_timeout: Duration::from_millis(retry_timeout_val),
            config,
            role,
            waypoint,
            peer_manager: PeerManager::new(upstream_peers),
            subscriptions: HashMap::new(),
            sync_request: None,
            initialization_listener: None,
            executor_proxy,
        }
    }

    /// main routine. starts sync coordinator that listens for CoordinatorMsg
    pub async fn start(mut self, network: Vec<(StateSynchronizerSender, StateSynchronizerEvents)>) {
        let mut interval = interval(Duration::from_millis(self.config.tick_interval_ms)).fuse();

        let network_senders: Vec<StateSynchronizerSender> =
            network.iter().map(|t| t.0.clone()).collect();
        let events: Vec<_> = network
            .into_iter()
            .enumerate()
            .map(|(idx, t)| t.1.map(move |e| (idx, e)))
            .collect();
        let mut network_events = select_all(events).fuse();

        loop {
            ::futures::select! {
                msg = self.client_events.select_next_some() => {
                    match msg {
                        CoordinatorMessage::Request(request) => {
                            if let Err(e) = self.request_sync(request) {
                                error!("[state sync] request sync fail: {}", e);
                            }
                        }
                        CoordinatorMessage::Commit(txns, events, callback) => {
                            if let Err(e) = self.process_commit(txns, Some(callback)).await {
                                error!("[state sync] process commit fail: {}", e);
                            }
                            if let Err(e) = self.executor_proxy.publish_on_chain_config_updates(events){
                                error!("[state sync] failed to publish reconfig notification: {}", e);
                            }
                        }
                        CoordinatorMessage::GetState(callback) => {
                            self.get_state(callback);
                        }
                        CoordinatorMessage::GetEpochProof(request) => {
                            self.get_epoch_proof(request);
                        }
                        CoordinatorMessage::WaitInitialize(cb_sender) => {
                            self.set_initialization_listener(cb_sender);
                        }
                    };
                },
                (idx, network_event) = network_events.select_next_some() => {
                    match network_event {
                        Ok(event) => {
                            match event {
                                Event::NewPeer(peer_id) => {
                                    debug!("[state sync] new peer {}", peer_id);
                                    self.peer_manager.enable_peer(peer_id, network_senders[idx].clone());
                                    self.check_progress();
                                }
                                Event::LostPeer(peer_id) => {
                                    debug!("[state sync] lost peer {}", peer_id);
                                    self.peer_manager.disable_peer(&peer_id);
                                }
                                Event::Message((peer_id, mut message)) => self.process_one_message(peer_id, message).await,
                                _ => warn!("[state sync] unexpected event: {:?}", event),
                            }
                        },
                        Err(err) => { error!("[state sync] network error {}", err); },
                    }
                },
                _ = interval.select_next_some() => {
                    self.check_progress();
                }
            }
        }
    }

    async fn process_one_message(&mut self, peer_id: PeerId, msg: StateSynchronizerMsg) {
        match msg {
            StateSynchronizerMsg::GetChunkRequest(request) => {
                if let Err(err) = self.process_chunk_request(peer_id, *request) {
                    error!("[state sync] failed to serve chunk request from {}, local LI version {}: {}", peer_id, self.local_state.highest_local_li.ledger_info().version(), err);
                }
            }
            StateSynchronizerMsg::GetChunkResponse(response) => {
                if let Err(err) = self.process_chunk_response(&peer_id, *response).await {
                    error!(
                        "[state sync] failed to process chunk response from {}: {}",
                        peer_id, err
                    );
                    counters::APPLY_CHUNK_FAILURE
                        .with_label_values(&[&*peer_id.to_string()])
                        .inc();
                } else {
                    self.peer_manager
                        .update_score(&peer_id, PeerScoreUpdateType::Success);
                    counters::APPLY_CHUNK_SUCCESS
                        .with_label_values(&[&*peer_id.to_string()])
                        .inc();
                }
            }
        }
    }

    /// Sync up coordinator state with the local storage.
    fn sync_state_with_local_storage(&mut self) -> Result<()> {
        let new_state = self.executor_proxy.get_local_storage_state()?;
        if new_state.epoch() > self.local_state.epoch() {
            debug!(
                "[state sync] Trusted epoch moved from {} to {}",
                self.local_state.epoch(),
                new_state.epoch()
            );
        }
        self.local_state = new_state;
        Ok(())
    }

    /// In case waypoint is set verify that the local LI has reached the waypoint version.
    fn is_initialized(&self) -> bool {
        self.waypoint.as_ref().map_or(true, |w| {
            w.version() <= self.local_state.highest_local_li.ledger_info().version()
        })
    }

    fn set_initialization_listener(&mut self, cb_sender: oneshot::Sender<Result<()>>) {
        if self.is_initialized() {
            if cb_sender.send(Ok(())).is_err() {
                error!("Error sending initialization notification");
            }
        } else {
            self.initialization_listener = Some(cb_sender);
        }
    }

    /// In case there has been another pending request it's going to be overridden.
    /// The caller will be notified about request completion via request.callback oneshot:
    /// at that moment it's guaranteed that the highest LI exposed by the storage is equal to the
    /// target LI.
    /// StateSynchronizer assumes that it's the only one modifying the storage (consensus is not
    /// trying to commit transactions concurrently).
    fn request_sync(&mut self, request: SyncRequest) -> Result<()> {
        self.sync_state_with_local_storage()?;
        ensure!(
            self.is_initialized(),
            "[state sync] Sync request but initialization is not complete!"
        );
        let highest_local_li = self.local_state.highest_local_li.ledger_info();
        let target_version = request.target.ledger_info().version();
        if target_version == highest_local_li.version() {
            return request
                .callback
                .send(Ok(()))
                .map_err(|_| format_err!("Callback error"));
        }

        if target_version < highest_local_li.version() {
            request
                .callback
                .send(Err(format_err!("Sync request to old version")))
                .map_err(|_| format_err!("Callback error"))?;
            bail!(
                "[state sync] Sync request for version {} < known version {}",
                target_version,
                highest_local_li.version()
            );
        }
        counters::TARGET_VERSION.set(target_version as i64);
        debug!(
            "[state sync] sync requested. Known LI: {}, requested_version: {}",
            highest_local_li, target_version
        );

        self.peer_manager
            .set_peers(request.target.signatures().keys().copied().collect());
        self.sync_request = Some(request);
        self.send_chunk_request(
            self.local_state.highest_version_in_local_storage(),
            self.local_state.epoch(),
        )
    }

    /// The function is called after new txns have been applied to the local storage.
    /// As a result it might:
    /// 1) help remote subscribers with long poll requests, 2) finish local sync request
    async fn process_commit(
        &mut self,
        transactions: Vec<Transaction>,
        commit_callback: Option<oneshot::Sender<Result<CommitResponse>>>,
    ) -> Result<()> {
        // We choose to re-sync the state with the storage as it's the simplest approach:
        // in case the performance implications of re-syncing upon every commit are high,
        // it's possible to manage some of the highest known versions in memory.
        self.sync_state_with_local_storage()?;
        let local_version = self.local_state.highest_version_in_local_storage();
        counters::COMMITTED_VERSION.set(local_version as i64);
        let block_timestamp_usecs = self
            .local_state
            .highest_local_li
            .ledger_info()
            .timestamp_usecs();

        // send notif to shared mempool
        // filter for user transactions here
        let mut committed_user_txns = vec![];
        for txn in transactions {
            if let Transaction::UserTransaction(signed_txn) = txn {
                committed_user_txns.push(CommittedTransaction {
                    sender: signed_txn.sender(),
                    sequence_number: signed_txn.sequence_number(),
                });
            }
        }
        let (callback, callback_rcv) = oneshot::channel();
        let req = CommitNotification {
            transactions: committed_user_txns,
            block_timestamp_usecs,
            callback,
        };
        let mut mempool_channel = self.state_sync_to_mempool_sender.clone();
        let mut msg = "";
        if let Err(e) = mempool_channel.try_send(req) {
            error!(
                "[state sync] failed to send commit notif to shared mempool: {:?}",
                e
            );
            msg = "state sync failed to send commit notif to shared mempool";
        }
        if let Err(e) = timeout(Duration::from_secs(1), callback_rcv).await {
            error!(
                "[state sync] did not receive ACK for commit notification sent to mempool: {:?}",
                e
            );
            msg = "state sync did not receive ACK for commit notification sent to mempool";
        }

        if let Some(cb) = commit_callback {
            // send back ACK to consensus
            if let Err(e) = cb.send(Ok(CommitResponse {
                msg: msg.to_string(),
            })) {
                error!(
                    "[state sync] failed to send commit ACK back to consensus: {:?}",
                    e
                );
            }
        }

        self.check_subscriptions();
        self.peer_manager.remove_requests(local_version);

        let sync_request_complete = self.sync_request.as_ref().map_or(false, |sync_req| {
            // Each `ChunkResponse` is verified to make sure it never goes beyond the requested
            // target version, hence, the local version should never go beyond sync req target.
            assert!(local_version <= sync_req.target.ledger_info().version());
            sync_req.target.ledger_info().version() == local_version
        });

        if sync_request_complete {
            debug!(
                "[state sync] synchronization to {} is finished",
                local_version
            );
            if let Some(sync_request) = self.sync_request.take() {
                sync_request
                    .callback
                    .send(Ok(()))
                    .map_err(|_| format_err!("Callback error"))?;
            }
        }

        let initialization_complete = self
            .initialization_listener
            .as_ref()
            .map_or(false, |_| self.is_initialized());
        if initialization_complete {
            debug!(
                "[state sync] Finished initialization to waypoint. Current version: {}",
                self.local_state.highest_local_li.ledger_info().version()
            );
            if let Some(listener) = self.initialization_listener.take() {
                listener
                    .send(Ok(()))
                    .map_err(|_| format_err!("Error sending initialization notification"))?;
            }
        }
        Ok(())
    }

    fn get_state(&self, callback: oneshot::Sender<SynchronizerState>) {
        if callback.send(self.local_state.clone()).is_err() {
            error!("[state sync] failed to send internal state");
        }
    }

    /// There are two types of ChunkRequests:
    /// 1) Validator chunk requests are for a specific target LI and don't ask for long polling.
    /// 2) FullNode chunk requests don't specify a target LI and can allow long polling.
    fn process_chunk_request(&mut self, peer_id: PeerId, request: GetChunkRequest) -> Result<()> {
        self.sync_state_with_local_storage()?;
        debug!(
            "[state sync] chunk request: peer_id: {}, local li version: {}, req: {}",
            peer_id.short_str(),
            self.local_state.highest_local_li.ledger_info().version(),
            request,
        );

        let sender = self
            .peer_manager
            .get_network_sender(&peer_id)
            .ok_or_else(|| format_err!("ChunkRequest from unknown peer {}", peer_id.short_str()))?;
        match request.target().clone() {
            TargetType::TargetLedgerInfo(li) => {
                self.process_request_target_li(sender, peer_id, request, li)
            }
            TargetType::HighestAvailable { timeout_ms } => {
                self.process_request_highest_available(sender, peer_id, request, timeout_ms)
            }
            TargetType::Waypoint(waypoint_version) => {
                self.process_request_waypoint(sender, peer_id, request, waypoint_version)
            }
        }
    }

    /// Processing requests with a specified target LedgerInfo.
    /// Assumes that the local state is uptodate with storage.
    fn process_request_target_li(
        &self,
        sender: StateSynchronizerSender,
        peer_id: PeerId,
        request: GetChunkRequest,
        target_li: LedgerInfoWithSignatures,
    ) -> Result<()> {
        let limit = std::cmp::min(request.limit, self.config.max_chunk_limit);
        let response_li = self.choose_response_li(
            request.known_version,
            request.current_epoch,
            Some(target_li),
        )?;
        // In case known_version is lower than the requested ledger info an empty response might be
        // sent.
        self.deliver_chunk(
            peer_id,
            request.known_version,
            ResponseLedgerInfo::VerifiableLedgerInfo(response_li),
            limit,
            sender,
        )
    }

    /// Processing requests with no target LedgerInfo (highest available) and potentially long
    /// polling.
    /// Assumes that the local state is uptodate with storage.
    fn process_request_highest_available(
        &mut self,
        sender: StateSynchronizerSender,
        peer_id: PeerId,
        request: GetChunkRequest,
        timeout_ms: u64,
    ) -> Result<()> {
        let limit = std::cmp::min(request.limit, self.config.max_chunk_limit);
        let timeout = std::cmp::min(timeout_ms, self.config.max_timeout_ms);

        let response_li =
            self.choose_response_li(request.known_version, request.current_epoch, None)?;
        // If there is nothing a node can help with, and the request supports long polling,
        // add it to the subscriptions.
        if self.local_state.highest_local_li.ledger_info().version() <= request.known_version
            && timeout > 0
        {
            let expiration_time = SystemTime::now().checked_add(Duration::from_millis(timeout));
            if let Some(time) = expiration_time {
                let request_info = PendingRequestInfo {
                    expiration_time: time,
                    known_version: request.known_version,
                    request_epoch: request.current_epoch,
                    limit,
                };
                self.subscriptions.insert(peer_id, request_info);
            }
            return Ok(());
        }

        self.deliver_chunk(
            peer_id,
            request.known_version,
            ResponseLedgerInfo::VerifiableLedgerInfo(response_li),
            limit,
            sender,
        )
    }

    fn process_request_waypoint(
        &self,
        sender: StateSynchronizerSender,
        peer_id: PeerId,
        request: GetChunkRequest,
        waypoint_version: Version,
    ) -> Result<()> {
        let mut limit = std::cmp::min(request.limit, self.config.max_chunk_limit);
        ensure!(
            self.local_state.highest_local_li.ledger_info().version() >= waypoint_version,
            "Local version {} < requested waypoint version {}.",
            self.local_state.highest_local_li.ledger_info().version(),
            waypoint_version
        );
        ensure!(
            request.known_version < waypoint_version,
            "Waypoint request version {} is not smaller than waypoint {}",
            request.known_version,
            waypoint_version
        );

        // Retrieve the waypoint LI.
        let waypoint_li = self.executor_proxy.get_ledger_info(waypoint_version)?;

        // Txns are up to the end of request epoch with the proofs relative to the waypoint LI.
        let end_of_epoch_li = if waypoint_li.ledger_info().epoch() > request.current_epoch {
            Some(
                self.executor_proxy
                    .get_epoch_proof(request.current_epoch, request.current_epoch + 1)?
                    .ledger_info_with_sigs
                    .first()
                    .ok_or_else(|| {
                        format_err!(
                            "No end of epoch LedgerInfo found for epoch {}",
                            request.current_epoch
                        )
                    })?
                    .clone(),
            )
        } else {
            None
        };
        if let Some(li) = end_of_epoch_li.as_ref() {
            let num_txns_until_end_of_epoch = li.ledger_info().version() - request.known_version;
            limit = std::cmp::min(limit, num_txns_until_end_of_epoch);
        }
        self.deliver_chunk(
            peer_id,
            request.known_version,
            ResponseLedgerInfo::LedgerInfoForWaypoint {
                waypoint_li,
                end_of_epoch_li,
            },
            limit,
            sender,
        )
    }

    /// Generate and send the ChunkResponse to the given peer.
    /// The chunk response contains transactions from the local storage with the proofs relative to
    /// the given target ledger info.
    /// In case target is None, the ledger info is set to the local highest ledger info.
    fn deliver_chunk(
        &self,
        peer_id: PeerId,
        known_version: u64,
        response_li: ResponseLedgerInfo,
        limit: u64,
        mut network_sender: StateSynchronizerSender,
    ) -> Result<()> {
        let txns = self
            .executor_proxy
            .get_chunk(known_version, limit, response_li.version())?;
        let chunk_response = GetChunkResponse::new(response_li, txns);
        let msg = StateSynchronizerMsg::GetChunkResponse(Box::new(chunk_response));
        if network_sender.send_to(peer_id, msg).is_err() {
            error!("[state sync] failed to send p2p message");
        }
        Ok(())
    }

    /// The choice of the LedgerInfo in the response follows the following logic:
    /// * response LI is either the requested target or the highest local LI if target is None.
    /// * if the response LI would not belong to `request_epoch`, change
    /// the response LI to the LI that is terminating `request_epoch`.
    fn choose_response_li(
        &self,
        known_version: u64,
        request_epoch: u64,
        target: Option<LedgerInfoWithSignatures>,
    ) -> Result<LedgerInfoWithSignatures> {
        let mut target_li = target.unwrap_or_else(|| self.local_state.highest_local_li.clone());
        if target_li.ledger_info().epoch() > request_epoch {
            let end_of_epoch_li = self
                .executor_proxy
                .get_epoch_proof(request_epoch, request_epoch + 1)?
                .ledger_info_with_sigs
                .first()
                .ok_or_else(|| {
                    format_err!(
                        "[state sync] Fail to retrieve end of epoch LI for epoch {}",
                        request_epoch
                    )
                })?
                .clone();
            debug!("[state sync] Chunk response for known_version = {} is limited to the last txn of epoch {} at version {}", known_version, request_epoch, end_of_epoch_li.ledger_info().version());
            target_li = end_of_epoch_li;
        }
        Ok(target_li)
    }

    /// * Issue a request for the next chunk.
    /// * Validate and execute the transactions.
    /// * Notify the clients in case a sync request has been completed.
    async fn process_chunk_response(
        &mut self,
        peer_id: &PeerId,
        response: GetChunkResponse,
    ) -> Result<()> {
        counters::RESPONSES_RECEIVED
            .with_label_values(&[&*peer_id.to_string()])
            .inc();
        debug!("[state sync] Processing chunk response {}", response);
        let txn_list_with_proof = response.txn_list_with_proof.clone();
        let known_version = self.local_state.highest_version_in_local_storage();
        let chunk_start_version =
            txn_list_with_proof
                .first_transaction_version
                .ok_or_else(|| {
                    self.peer_manager
                        .update_score(&peer_id, PeerScoreUpdateType::EmptyChunk);
                    format_err!("[state sync] Empty chunk from {}", peer_id.short_str())
                })?;

        if chunk_start_version != known_version + 1 {
            // Old / wrong chunk.
            self.peer_manager
                .update_score(&peer_id, PeerScoreUpdateType::ChunkVersionCannotBeApplied);
            bail!(
                "[state sync] Non sequential chunk from {}: known_version: {}, received: {}",
                peer_id.short_str(),
                known_version,
                chunk_start_version
            );
        }

        let chunk_size = txn_list_with_proof.len() as u64;
        let new_version = known_version + chunk_size;
        match response.response_li {
            ResponseLedgerInfo::VerifiableLedgerInfo(li) => {
                self.process_response_with_verifiable_li(txn_list_with_proof, li)
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
            self.peer_manager
                .update_score(peer_id, PeerScoreUpdateType::InvalidChunk);
            format_err!("[state sync] failed to apply chunk: {}", e)
        })?;

        counters::STATE_SYNC_TXN_REPLAYED.inc_by(chunk_size as i64);
        debug!(
            "[state sync] applied chunk. Previous version: {}, new version: {}, chunk size: {}",
            known_version, new_version, chunk_size
        );

        // The overall chunk processing duration is calculated starting from the very first attempt
        // until the commit
        if let Some(first_attempt_tst) = self.peer_manager.get_first_request_time(known_version + 1)
        {
            if let Ok(duration) = SystemTime::now().duration_since(first_attempt_tst) {
                counters::SYNC_PROGRESS_DURATION.observe_duration(duration);
            }
        }

        self.process_commit(response.txn_list_with_proof.transactions, None)
            .await
    }

    /// Processing chunk responses that carry a LedgerInfo that should be verified using the
    /// current local trusted validator set.
    fn process_response_with_verifiable_li(
        &mut self,
        txn_list_with_proof: TransactionListWithProof,
        response_li: LedgerInfoWithSignatures,
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
        let new_version =
            self.local_state.highest_version_in_local_storage() + txn_list_with_proof.len() as u64;
        let new_epoch = if response_li.ledger_info().version() == new_version
            && response_li.ledger_info().next_validator_set().is_some()
        {
            // This chunk is going to finish the current epoch, optimistically request a chunk
            // from the next epoch.
            self.local_state.epoch() + 1
        } else {
            // Remain in the current epoch
            self.local_state.epoch()
        };
        self.send_chunk_request(new_version, new_epoch)?;
        let verifier = VerifierType::TrustedVerifier(self.local_state.trusted_epoch.clone());
        verifier.verify(&response_li)?;
        self.validate_and_store_chunk(txn_list_with_proof, response_li, None)
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
        let new_version =
            self.local_state.highest_version_in_local_storage() + txn_list_with_proof.len() as u64;
        // The epoch in the optimistic request should be the next epoch if the current chunk
        // is the last one in its epoch.
        let new_epoch = end_of_epoch_li
            .as_ref()
            .map_or(self.local_state.epoch(), |li| {
                if li.ledger_info().version() == new_version {
                    self.local_state.epoch() + 1
                } else {
                    self.local_state.epoch()
                }
            });
        if new_version < self.waypoint.as_ref().map_or(0, |w| w.version()) {
            self.send_chunk_request(new_version, new_epoch)?;
        }

        self.waypoint
            .as_ref()
            .ok_or_else(|| {
                format_err!("No waypoint found to process a response with a waypoint LI")
            })
            .and_then(|w| w.verify(waypoint_li.ledger_info()))?;
        self.validate_and_store_chunk(txn_list_with_proof, waypoint_li, end_of_epoch_li)
    }

    // Assumes that the target LI has been already verified by the caller.
    fn validate_and_store_chunk(
        &mut self,
        txn_list_with_proof: TransactionListWithProof,
        target: LedgerInfoWithSignatures,
        intermediate_end_of_epoch_li: Option<LedgerInfoWithSignatures>,
    ) -> Result<()> {
        let target_epoch_and_round = (target.ledger_info().epoch(), target.ledger_info().round());
        let local_epoch_and_round = (
            self.local_state.highest_local_li.ledger_info().epoch(),
            self.local_state.highest_local_li.ledger_info().round(),
        );
        if target_epoch_and_round < local_epoch_and_round {
            warn!(
                "Ledger info is too old: local epoch/round: {:?}, epoch/round in request: {:?}.",
                local_epoch_and_round, target_epoch_and_round,
            );
            return Ok(());
        }

        self.executor_proxy.execute_chunk(
            txn_list_with_proof,
            target,
            intermediate_end_of_epoch_li,
            &mut self.local_state.synced_trees,
        )?;
        Ok(())
    }

    /// Ensures that StateSynchronizer is making progress:
    /// issue a new request if too much time passed since requesting highest_committed_version + 1.
    fn check_progress(&mut self) {
        if self.peer_manager.is_empty() {
            return;
        }
        if self.role == RoleType::Validator && self.sync_request.is_none() && self.is_initialized()
        {
            return;
        }

        let known_version = self.local_state.highest_version_in_local_storage();
        let last_request_tst = self
            .peer_manager
            .get_last_request_time(known_version + 1)
            .unwrap_or(UNIX_EPOCH);

        // if coordinator didn't make progress by expected time, issue new request
        if let Some(tst) = last_request_tst.checked_add(self.retry_timeout) {
            if SystemTime::now().duration_since(tst).is_ok() {
                self.peer_manager
                    .process_timeout(known_version + 1, self.role == RoleType::Validator);
                if let Err(e) = self.send_chunk_request(known_version, self.local_state.epoch()) {
                    error!("[state sync] Failed to send chunk request: {}", e);
                }
                counters::TIMEOUT.inc();
            }
        }
    }

    /// Sends a chunk request with a given `known_version` and `known_epoch`
    /// (might be chosen optimistically).
    /// The request includes a target for Validator and a non-zero timeout for a FullNode.
    fn send_chunk_request(&mut self, known_version: u64, known_epoch: u64) -> Result<()> {
        let (peer_id, mut sender) = self
            .peer_manager
            .pick_peer()
            .ok_or_else(|| format_err!("No peers found for chunk request."))?;

        let target = if !self.is_initialized() {
            let waypoint_version =
                self.waypoint.as_ref().map(|w| w.version()).ok_or_else(|| {
                    format_err!("No waypoint found but coordinator is not initialized.")
                })?;
            TargetType::Waypoint(waypoint_version)
        } else {
            match self.sync_request.as_ref() {
                None => TargetType::HighestAvailable {
                    timeout_ms: self.config.long_poll_timeout_ms,
                },
                Some(sync_req) => {
                    if sync_req.target.ledger_info().version() <= known_version {
                        debug!(
                            "[state sync] Reached version {}, no need to send more requests",
                            known_version
                        );
                        return Ok(());
                    }
                    TargetType::TargetLedgerInfo(sync_req.target.clone())
                }
            }
        };

        let req = GetChunkRequest::new(known_version, known_epoch, self.config.chunk_limit, target);
        debug!(
            "[state sync] request next chunk. peer_id: {}, chunk req: {}",
            peer_id.short_str(),
            req,
        );
        let msg = StateSynchronizerMsg::GetChunkRequest(Box::new(req));
        self.peer_manager
            .process_request(known_version + 1, peer_id);
        sender.send_to(peer_id, msg)?;
        counters::REQUESTS_SENT
            .with_label_values(&[&*peer_id.to_string()])
            .inc();
        Ok(())
    }

    fn deliver_subscription(
        &self,
        peer_id: PeerId,
        sender: StateSynchronizerSender,
        request_info: PendingRequestInfo,
    ) -> Result<()> {
        let response_li =
            self.choose_response_li(request_info.known_version, request_info.request_epoch, None)?;
        self.deliver_chunk(
            peer_id,
            request_info.known_version,
            ResponseLedgerInfo::VerifiableLedgerInfo(response_li),
            request_info.limit,
            sender,
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
        let highest_li_version = self.local_state.highest_local_li.ledger_info().version();

        let mut ready = vec![];
        self.subscriptions.retain(|peer_id, request_info| {
            // filter out expired peer requests
            if SystemTime::now()
                .duration_since(request_info.expiration_time.clone())
                .is_ok()
            {
                return false;
            }
            if request_info.known_version < highest_li_version {
                ready.push((*peer_id, request_info.clone()));
                false
            } else {
                true
            }
        });

        ready.into_iter().for_each(|(peer_id, request_info)| {
            if let Some(sender) = self.peer_manager.get_network_sender(&peer_id) {
                if let Err(err) = self.deliver_subscription(peer_id, sender, request_info) {
                    error!("[state sync] failed to notify subscriber {}", err);
                }
            }
        });
    }

    fn get_epoch_proof(&self, request: EpochRetrievalRequest) {
        if request
            .callback
            .send(
                self.executor_proxy
                    .get_epoch_proof(request.start_epoch, request.end_epoch),
            )
            .is_err()
        {
            error!("[state sync] coordinator failed to send back epoch proof");
        }
    }
}
