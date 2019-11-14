// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    counters,
    executor_proxy::ExecutorProxyTrait,
    peer_manager::{PeerManager, PeerScoreUpdateType},
    PeerId, SynchronizerState,
};
use failure::prelude::*;
use futures::{
    channel::{mpsc, oneshot},
    stream::{futures_unordered::FuturesUnordered, select_all},
    StreamExt,
};
use libra_config::config::RoleType;
use libra_config::config::StateSyncConfig;
use libra_logger::prelude::*;
use libra_types::crypto_proxies::ValidatorChangeEventWithProof;
use libra_types::{
    crypto_proxies::LedgerInfoWithSignatures, transaction::TransactionListWithProof,
};
use network::{
    proto::{GetChunkRequest, GetChunkResponse, StateSynchronizerMsg, StateSynchronizerMsg_oneof},
    validator_network::{Event, StateSynchronizerEvents, StateSynchronizerSender},
};
use std::{
    collections::HashMap,
    convert::TryInto,
    str::FromStr,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::timer::Interval;

pub(crate) struct SyncRequest {
    // The Result value returned to the caller is Error in case the StateSynchronizer failed to
    // reach the target (the LI in the storage remains unchanged as if nothing happened).
    pub callback: oneshot::Sender<Result<()>>,
    pub target: LedgerInfoWithSignatures,
}

pub(crate) struct EpochRetrievalRequest {
    pub start_epoch: u64,
    pub callback: oneshot::Sender<Result<ValidatorChangeEventWithProof>>,
}

/// message used by StateSyncClient for communication with Coordinator
pub(crate) enum CoordinatorMessage {
    // used to initiate new sync
    Request(SyncRequest),
    // used to notify about new txn commit
    Commit,
    GetState(oneshot::Sender<SynchronizerState>),
    // used to generate epoch proof
    GetEpochProof(EpochRetrievalRequest),
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
    // peers used for synchronization
    peer_manager: PeerManager,
    // Optional sync request to be called when the target sync is reached
    sync_request: Option<SyncRequest>,
    // queue of incoming long polling requests
    // peer will be notified about new chunk of transactions if it's available before expiry time
    // value format is (expiration_time, known_version, limit)
    subscriptions: HashMap<PeerId, (SystemTime, u64, u64)>,
    executor_proxy: T,
}

impl<T: ExecutorProxyTrait> SyncCoordinator<T> {
    pub fn new(
        client_events: mpsc::UnboundedReceiver<CoordinatorMessage>,
        role: RoleType,
        config: StateSyncConfig,
        executor_proxy: T,
    ) -> Self {
        let upstream_peers: Vec<_> = config
            .upstream_peers
            .upstream_peers
            .iter()
            .map(|peer_id_str| {
                PeerId::from_str(peer_id_str).unwrap_or_else(|_| {
                    // invariant of UpstreamPeersConfig
                    unreachable!("Failed to parse peer_id from string: {}", peer_id_str)
                })
            })
            .collect();
        let retry_timeout_val = match role {
            RoleType::FullNode => config.tick_interval_ms + config.long_poll_timeout_ms,
            RoleType::Validator => 2 * config.tick_interval_ms,
        };

        // The uptodate sync state is retrieved from the local storage asynchronously, hence
        // we're going to initialize with an empty state and retrieve it upon start() only.
        Self {
            client_events,
            local_state: SynchronizerState::zero_state(),
            retry_timeout: Duration::from_millis(retry_timeout_val),
            config,
            role,
            peer_manager: PeerManager::new(upstream_peers),
            subscriptions: HashMap::new(),
            sync_request: None,
            executor_proxy,
        }
    }

    /// main routine. starts sync coordinator that listens for CoordinatorMsg
    pub async fn start(mut self, network: Vec<(StateSynchronizerSender, StateSynchronizerEvents)>) {
        self.sync_state_with_local_storage()
            .await
            .expect("[state sync] Start failure: cannot sync with storage.");

        let mut interval =
            Interval::new_interval(Duration::from_millis(self.config.tick_interval_ms)).fuse();

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
                            if let Err(e) = self.request_sync(request).await {
                                error!("[state sync] request sync fail: {}", e);
                            }
                        }
                        CoordinatorMessage::Commit => {
                            if let Err(e) = self.process_commit().await {
                                error!("[state sync] process commit fail: {}", e);
                            }
                        }
                        CoordinatorMessage::GetState(callback) => {
                            self.get_state(callback);
                        }
                        CoordinatorMessage::GetEpochProof(request) => {
                            self.get_epoch_proof(request).await;
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
                                    self.check_progress().await;
                                }
                                Event::LostPeer(peer_id) => {
                                    debug!("[state sync] lost peer {}", peer_id);
                                    self.peer_manager.disable_peer(&peer_id);
                                }
                                Event::Message((peer_id, mut message)) => {
                                    match message.message.unwrap() {
                                        StateSynchronizerMsg_oneof::ChunkRequest(request) => {
                                            if let Err(err) = self.process_chunk_request(peer_id, request).await {
                                                error!("[state sync] failed to serve chunk request from {}, local LI version {}: {}", peer_id, self.local_state.highest_local_li.ledger_info().version(), err);
                                            }
                                        }
                                        StateSynchronizerMsg_oneof::ChunkResponse(response) => {
                                            if let Err(err) = self.process_chunk_response(&peer_id, response).await {
                                                error!("[state sync] failed to process chunk response from {}: {}", peer_id, err);
                                                counters::APPLY_CHUNK_FAILURE.with_label_values(&[&*peer_id.to_string()]).inc();
                                            } else {
                                                self.peer_manager.update_score(&peer_id, PeerScoreUpdateType::Success);
                                                counters::APPLY_CHUNK_SUCCESS.with_label_values(&[&*peer_id.to_string()]).inc();
                                            }
                                        }
                                    }
                                }
                                _ => {}
                            }
                        },
                        Err(err) => { error!("[state sync] network error {}", err); },
                    }
                },
                _ = interval.select_next_some() => {
                    self.check_progress().await;
                }
            }
        }
    }

    /// Sync up coordinator state with the local storage.
    async fn sync_state_with_local_storage(&mut self) -> Result<()> {
        self.local_state = self.executor_proxy.get_local_storage_state().await?;
        Ok(())
    }

    /// In case there has been another pending request it's going to be overridden.
    /// The caller will be notified about request completion via request.callback oneshot:
    /// at that moment it's guaranteed that the highest LI exposed by the storage is equal to the
    /// target LI.
    /// StateSynchronizer assumes that it's the only one modifying the storage (consensus is not
    /// trying to commit transactions concurrently).
    async fn request_sync(&mut self, request: SyncRequest) -> Result<()> {
        self.sync_state_with_local_storage().await?;
        let highest_local_li = self.local_state.highest_local_li.ledger_info();
        let target_version = request.target.ledger_info().version();
        counters::TARGET_VERSION.set(target_version as i64);
        debug!(
            "[state sync] sync requested. Known LI: {}, requested_version: {}",
            highest_local_li, target_version
        );

        if target_version <= highest_local_li.version() {
            request
                .callback
                .send(Err(format_err!("Sync request for an old version")))
                .map_err(|_| format_err!("Callback error"))?;
            bail!(
                "[state sync] Sync request for version {} <= known version {}",
                target_version,
                highest_local_li.version()
            );
        }

        self.peer_manager
            .set_peers(request.target.signatures().keys().copied().collect());
        self.sync_request = Some(request);
        self.send_chunk_request(self.local_state.highest_version_in_local_storage())
            .await
    }

    /// The function is called after new txns have been applied to the local storage.
    /// As a result it might:
    /// 1) help remote subscribers with long poll requests, 2) finish local sync request
    async fn process_commit(&mut self) -> Result<()> {
        // We choose to re-sync the state with the storage as it's the simplest approach:
        // in case the performance implications of re-syncing upon every commit are high,
        // it's possible to manage some of the highest known versions in memory.
        self.sync_state_with_local_storage().await?;
        let local_version = self.local_state.highest_version_in_local_storage();
        counters::COMMITTED_VERSION.set(local_version as i64);

        self.check_subscriptions().await;
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
    async fn process_chunk_request(
        &mut self,
        peer_id: PeerId,
        mut request: GetChunkRequest,
    ) -> Result<()> {
        if request.timeout > self.config.max_timeout_ms
            || request.limit > self.config.max_chunk_limit
        {
            bail!(
                "[state sync] Request timeout: {}, chunk limit: {}; configured max timeout is {} ms, and chunk limit is {}",
                request.timeout,
                request.limit,
                self.config.max_timeout_ms,
                self.config.max_chunk_limit
            );
        }
        let target = request
            .ledger_info_with_sigs
            .take()
            .map(TryInto::try_into)
            .transpose()?;
        self.sync_state_with_local_storage().await?;
        let local_li_version = self.local_state.highest_local_li.ledger_info().version();
        debug!(
            "[state sync] chunk request: peer_id: {}, request known version: {}, target version: {}, local li version: {}",
            peer_id.short_str(),
            request.known_version,
            target.as_ref().map_or("None".to_string(), |t: &LedgerInfoWithSignatures| t.ledger_info().version().to_string()),
            local_li_version,
        );

        // If there is nothing a node can help with, and the request supports long polling,
        // add it to the subscriptions.
        if local_li_version <= request.known_version && request.timeout > 0 {
            let expiration_time =
                SystemTime::now().checked_add(Duration::from_millis(request.timeout));
            if let Some(time) = expiration_time {
                self.subscriptions
                    .insert(peer_id, (time, request.known_version, request.limit));
            }
            return Ok(());
        }

        // Send the chunk response right away (even if empty: empty response is better than no
        // response at all because it triggers another attempt without timing out).
        let sender = self
            .peer_manager
            .get_network_sender(&peer_id)
            .ok_or_else(|| format_err!("ChunkRequest from unknown peer {}", peer_id.short_str()))?;
        self.deliver_chunk(
            peer_id,
            request.known_version,
            request.limit,
            target,
            sender,
        )
        .await
    }

    /// Generate and send the ChunkResponse to the given peer.
    /// The chunk response contains transactions from the local storage with the proofs relative to
    /// the given target ledger info.
    /// In case target is None, the ledger info is set to the local highest ledger info.
    async fn deliver_chunk(
        &self,
        peer_id: PeerId,
        known_version: u64,
        limit: u64,
        target: Option<LedgerInfoWithSignatures>,
        mut network_sender: StateSynchronizerSender,
    ) -> Result<()> {
        // The LI in the response is either the given target or the highest LI of the node.
        let response_li = target.unwrap_or_else(|| self.local_state.highest_local_li.clone());
        let txns = self
            .executor_proxy
            .get_chunk(known_version, limit, response_li.ledger_info().version())
            .await?;
        let chunk_response = GetChunkResponse {
            ledger_info_with_sigs: Some(response_li.into()),
            txn_list_with_proof: Some(txns.into()),
        };
        let msg = StateSynchronizerMsg {
            message: Some(StateSynchronizerMsg_oneof::ChunkResponse(chunk_response)),
        };
        if network_sender.send_to(peer_id, msg).await.is_err() {
            error!("[state sync] failed to send p2p message");
        }
        Ok(())
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
        let txn_list_with_proof: TransactionListWithProof = response
            .txn_list_with_proof
            .ok_or_else(|| format_err!("Missing txn_list_with_proof"))?
            .try_into()?;

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
        let response_li: LedgerInfoWithSignatures = response
            .ledger_info_with_sigs
            .ok_or_else(|| format_err!("Missing ledger_info_with_sigs"))?
            .try_into()?;

        if let Some(sync_req) = self.sync_request.as_ref() {
            // Valid responses should not exceed the LI version of the request.
            if sync_req.target.ledger_info().version() < response_li.ledger_info().version() {
                self.peer_manager
                    .update_score(peer_id, PeerScoreUpdateType::InvalidChunk);
                bail!(
                    "[state sync] Response from {} has an LI version higher than requested.",
                    peer_id
                );
            }
        }

        let chunk_size = txn_list_with_proof.len() as u64;

        // Optimistically fetch the next chunk assuming the current chunk is going to be applied
        // successfully.
        let new_version = known_version + chunk_size;
        self.send_chunk_request(new_version).await?;

        self.validate_and_store_chunk(txn_list_with_proof, response_li)
            .await
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
        self.process_commit().await
    }

    async fn validate_and_store_chunk(
        &mut self,
        txn_list_with_proof: TransactionListWithProof,
        target: LedgerInfoWithSignatures,
    ) -> Result<()> {
        self.executor_proxy.validate_ledger_info(&target)?;
        self.executor_proxy
            .execute_chunk(txn_list_with_proof, target)
            .await?;
        Ok(())
    }

    /// Ensures that StateSynchronizer is making progress:
    /// issue a new request if too much time passed since requesting highest_committed_version + 1.
    async fn check_progress(&mut self) {
        if self.peer_manager.is_empty() {
            return;
        }
        if self.role == RoleType::Validator && self.sync_request.is_none() {
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
                if let Err(e) = self.send_chunk_request(known_version).await {
                    error!("[state sync] Failed to send chunk request: {}", e);
                }
                counters::TIMEOUT.inc();
            }
        }
    }

    /// Sends a chunk request with a given `known_version` (might be chosen optimistically).
    /// The request includes a target for Validator and a non-zero timeout for a FullNode.
    async fn send_chunk_request(&mut self, known_version: u64) -> Result<()> {
        let (peer_id, mut sender) = self
            .peer_manager
            .pick_peer()
            .ok_or_else(|| format_err!("No peers found for chunk request."))?;

        let mut req = GetChunkRequest::default();
        req.known_version = known_version;
        req.limit = self.config.chunk_limit;
        if self.role == RoleType::Validator {
            let target = self
                .sync_request
                .as_ref()
                .ok_or_else(|| {
                    format_err!("[state sync] Validator chunk request without a sync request.")
                })?
                .target
                .clone();
            if target.ledger_info().version() <= known_version {
                debug!(
                    "[state sync] Reached version {}, no need to send more requests",
                    known_version
                );
                return Ok(());
            }
            req.ledger_info_with_sigs = Some(target.into());
        } else {
            req.timeout = self.config.long_poll_timeout_ms;
        }
        debug!(
            "[state sync] request next chunk. peer_id: {}, known_version: {}, timeout: {}",
            peer_id.short_str(),
            known_version,
            req.timeout
        );
        let msg = StateSynchronizerMsg {
            message: Some(StateSynchronizerMsg_oneof::ChunkRequest(req)),
        };

        self.peer_manager
            .process_request(known_version + 1, peer_id);
        sender.send_to(peer_id, msg).await?;
        counters::REQUESTS_SENT
            .with_label_values(&[&*peer_id.to_string()])
            .inc();
        Ok(())
    }

    /// The function is called after the local storage is updated with new transactions:
    /// it might deliver chunks for the subscribers that have been waiting with the long polls.
    ///
    /// Note that it is possible to help the subscribers only with the transactions that match
    /// the highest ledger info in the local storage (some committed transactions are ahead of the
    /// latest ledger info and are not going to be used for helping the remote subscribers).
    /// The function assumes that the local state has been synced with storage.
    async fn check_subscriptions(&mut self) {
        let highest_li_version = self.local_state.highest_local_li.ledger_info().version();

        let mut ready = vec![];
        self.subscriptions
            .retain(|peer_id, (expiry, known_version, limit)| {
                // filter out expired peer requests
                if SystemTime::now().duration_since(expiry.clone()).is_ok() {
                    return false;
                }
                if *known_version < highest_li_version {
                    ready.push((*peer_id, *known_version, *limit));
                    false
                } else {
                    true
                }
            });

        let mut futures = FuturesUnordered::new();
        for (peer_id, known_version, limit) in ready {
            if let Some(sender) = self.peer_manager.get_network_sender(&peer_id) {
                futures.push(self.deliver_chunk(
                    peer_id,
                    known_version,
                    limit,
                    Some(self.local_state.highest_local_li.clone()),
                    sender,
                ));
            }
        }
        while let Some(res) = futures.next().await {
            if let Err(err) = res {
                error!("[state sync] failed to notify subscriber {}", err);
            }
        }
    }

    async fn get_epoch_proof(&self, request: EpochRetrievalRequest) {
        if request
            .callback
            .send(self.executor_proxy.get_epoch_proof(request.start_epoch))
            .is_err()
        {
            error!("[state sync] coordinator failed to send back epoch proof");
        }
    }
}
