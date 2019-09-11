// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    counters,
    executor_proxy::ExecutorProxyTrait,
    peer_manager::{PeerManager, PeerScoreUpdateType},
    LedgerInfo, PeerId,
};
use config::config::StateSyncConfig;
use crypto::ed25519::*;
use execution_proto::proto::execution::{ExecuteChunkRequest, ExecuteChunkResponse};
use failure::prelude::*;
use futures::{
    channel::{mpsc, oneshot},
    compat::Stream01CompatExt,
    stream::{futures_unordered::FuturesUnordered, select_all},
    StreamExt,
};
use logger::prelude::*;
use network::{
    proto::{GetChunkRequest, GetChunkResponse, StateSynchronizerMsg},
    validator_network::{Event, StateSynchronizerEvents, StateSynchronizerSender},
};
use proto_conv::{FromProto, IntoProto};
use std::{
    collections::HashMap,
    str::FromStr,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::timer::Interval;
use types::{ledger_info::LedgerInfoWithSignatures, proto::transaction::TransactionListWithProof};

/// message used by StateSyncClient for communication with Coordinator
pub enum CoordinatorMessage {
    // used to initiate new sync
    Requested(LedgerInfo, oneshot::Sender<bool>),
    // used to notify about new txn commit
    Commit(u64),
    GetState(oneshot::Sender<u64>),
}

/// used to coordinate synchronization process
/// handles external sync requests and drives synchronization with remote peers
pub(crate) struct SyncCoordinator<T> {
    // used to process client requests
    client_events: mpsc::UnboundedReceiver<CoordinatorMessage>,
    // last committed version that validator is aware of
    known_version: u64,
    // target state to sync to
    target: Option<LedgerInfo>,
    // config
    config: StateSyncConfig,
    // if autosync is on, StateSynchronizer will keep fetching chunks from upstream peers
    // even if no target state was specified
    autosync: bool,
    // peers used for synchronization. TBD: value is meta information about peer sync quality
    peer_manager: PeerManager,
    // option callback. Called when state sync reaches target version
    callback: Option<oneshot::Sender<bool>>,
    // timestamp of last commit
    last_commit: Option<SystemTime>,
    // queue of incoming long polling requests
    // peer will be notified about new chunk of transactions if it's available before expiry time
    // value format is (expiration_time, known_version, limit)
    subscriptions: HashMap<PeerId, (SystemTime, u64, u64)>,
    executor_proxy: T,
}

impl<T: ExecutorProxyTrait> SyncCoordinator<T> {
    pub fn new(
        client_events: mpsc::UnboundedReceiver<CoordinatorMessage>,
        config: StateSyncConfig,
        executor_proxy: T,
    ) -> Self {
        let upstream_peers: Vec<_> = config
            .upstream_peers
            .upstream_peers
            .iter()
            .map(|peer_id_str| {
                PeerId::from_str(peer_id_str).unwrap_or_else(|_| {
                    panic!("Failed to parse peer_id from string: {}", peer_id_str)
                })
            })
            .collect();
        Self {
            client_events,
            known_version: 0,
            target: None,
            config,
            // Note: We use upstream peer ids being non-empty as a proxy for a node being a full
            // node.
            autosync: !upstream_peers.is_empty(),
            peer_manager: PeerManager::new(upstream_peers),
            subscriptions: HashMap::new(),
            callback: None,
            last_commit: None,
            executor_proxy,
        }
    }

    /// main routine. starts sync coordinator that listens for CoordinatorMsg
    pub async fn start(mut self, network: Vec<(StateSynchronizerSender, StateSynchronizerEvents)>) {
        self.known_version = self
            .executor_proxy
            .get_latest_version()
            .await
            .expect("[start sync] failed to fetch latest version from storage");

        let mut interval =
            Interval::new_interval(Duration::from_millis(self.config.tick_interval_ms))
                .compat()
                .fuse();

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
                        CoordinatorMessage::Requested(target, subscription) => {
                            self.request_sync(target, subscription).await;
                        }
                        CoordinatorMessage::Commit(version) => {
                             self.commit(version).await;
                        }
                        CoordinatorMessage::GetState(callback) => {
                            self.get_state(callback);
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
                                    if message.has_chunk_request() {
                                        let known_version = message.get_chunk_request().get_known_version();
                                        if let Err(err) = self.process_chunk_request(peer_id, message.take_chunk_request()).await {
                                            error!("[state sync] failed to serve chunk request to {} with known version {}: {:?}", peer_id, known_version, err);
                                        }
                                    }
                                    if message.has_chunk_response() {
                                        if let Err(err) = self.process_chunk_response(&peer_id, message.take_chunk_response()).await {
                                            error!("[state sync] failed to process chunk response from {}: {:?}", peer_id, err);
                                        } else {
                                            self.peer_manager.update_score(&peer_id, PeerScoreUpdateType::Success);
                                        }
                                    }
                                }
                                _ => {}
                            }
                        },
                        Err(err) => { error!("[state sync] network error {:?}", err); },
                    }
                },
                _ = interval.select_next_some() => {
                    self.check_progress().await;
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

    async fn request_sync(&mut self, target: LedgerInfo, callback: oneshot::Sender<bool>) {
        let requested_version = target.ledger_info().version();
        self.known_version = self
            .executor_proxy
            .get_latest_version()
            .await
            .expect("[state sync] failed to fetch latest version from storage");

        debug!(
            "[state sync] sync requested. Known version: {}, requested_version: {}",
            self.known_version, requested_version
        );

        // if requested version equals to current committed, just pass ledger info to executor
        // there might be still empty blocks between committed state and requested
        if requested_version <= self.known_version {
            debug!("[state sync] sync contains only empty blocks");
            self.store_transactions(target.clone(), TransactionListWithProof::new())
                .await
                .expect("[state sync] failed to execute empty blocks");
            if callback.send(true).is_err() {
                error!("[state sync] coordinator failed to notify subscriber");
            }
            return;
        }

        // TODO: Should we be changing peer manager peer set for every target?
        self.peer_manager
            .set_peers(target.signatures().keys().copied().collect());
        self.target = Some(target);
        self.request_next_chunk(0).await;
        self.callback = Some(callback);
    }

    async fn commit(&mut self, version: u64) {
        debug!(
            "[state sync] commit. Known version: {}, version: {}",
            self.known_version, version
        );
        let is_update = version > self.known_version;
        self.known_version = std::cmp::max(version, self.known_version);
        if is_update {
            self.last_commit = Some(SystemTime::now());
            if let Err(err) = self.check_subscriptions().await {
                error!("[state sync] failed to check subscriptions: {:?}", err);
            }
        }
        if self.known_version == self.target_version() {
            debug!("[state sync] synchronization is finished");
            if let Some(cb) = self.callback.take() {
                if cb.send(true).is_err() {
                    error!("[state sync] failed to notify subscriber");
                }
            }
        }
    }

    fn get_state(&self, callback: oneshot::Sender<u64>) {
        if callback.send(self.known_version).is_err() {
            error!("[state sync] failed to fetch internal state");
        }
    }

    /// Get a batch of transactions
    async fn process_chunk_request(
        &mut self,
        peer_id: PeerId,
        mut request: GetChunkRequest,
    ) -> Result<()> {
        if request.timeout > self.config.max_timeout_ms
            || request.limit > self.config.max_chunk_limit
        {
            return Err(format_err!(
                "[state sync] timeout: {:?}, chunk limit: {:?}, but timeout must not exceed {:?} ms, and chunk limit must not exceed {:?}",
                request.timeout,
                request.limit,
                self.config.max_timeout_ms,
                self.config.max_chunk_limit
            ));
        }

        let latest_ledger_info = self.executor_proxy.get_latest_ledger_info().await?;
        let target = LedgerInfo::from_proto(request.take_ledger_info_with_sigs())
            .unwrap_or_else(|_| latest_ledger_info.clone());
        debug!("[state sync] chunk request: peer_id: {:?}, known_version: {}, latest_ledger_info: {}, target: {}", peer_id, request.known_version, latest_ledger_info.ledger_info().version(), target.ledger_info().version());

        // if upstream synchronizer doesn't have new data and request timeout is set
        // add peer request into subscription queue
        if self.known_version <= request.known_version && request.timeout > 0 {
            let expiration_time =
                SystemTime::now().checked_add(Duration::from_millis(request.timeout));
            if let Some(time) = expiration_time {
                self.subscriptions
                    .insert(peer_id, (time, request.known_version, request.limit));
            }
            Ok(())
        } else {
            match self.peer_manager.get_network_sender(&peer_id) {
                Some(sender) => {
                    self.deliver_chunk(
                        peer_id,
                        request.known_version,
                        request.limit,
                        target,
                        sender,
                    )
                    .await
                }
                None => Err(format_err!(
                    "[state sync] failed to find network for peer {}",
                    peer_id
                )),
            }
        }
    }

    async fn deliver_chunk(
        &self,
        peer_id: PeerId,
        known_version: u64,
        limit: u64,
        target: LedgerInfo,
        mut network_sender: StateSynchronizerSender,
    ) -> Result<()> {
        let response = self
            .executor_proxy
            .get_chunk(known_version, limit, target)
            .await?;
        let mut msg = StateSynchronizerMsg::new();
        msg.set_chunk_response(response);
        if network_sender.send_to(peer_id, msg).await.is_err() {
            error!("[state sync] failed to send p2p message");
        }
        Ok(())
    }

    /// processes batch of transactions downloaded from peer
    /// executes transactions, updates progress state, calls callback if some sync is finished
    async fn process_chunk_response(
        &mut self,
        peer_id: &PeerId,
        mut response: GetChunkResponse,
    ) -> Result<()> {
        let txn_list_with_proof = response.take_txn_list_with_proof();

        if let Some(version) = txn_list_with_proof
            .first_transaction_version
            .clone()
            .map(|x| x.get_value())
            .into_option()
        {
            // node has received a response from peer, so remove peer entry from requests map
            self.peer_manager.process_response(version, *peer_id);

            if version != self.known_version + 1 {
                return Err(format_err!(
                    "[state sync] non sequential chunk. Known version: {}, received: {}",
                    self.known_version,
                    version,
                ));
            }
        }

        let previous_version = self.known_version;
        let chunk_size = txn_list_with_proof.get_transactions().len();

        let result = self
            .validate_and_store_chunk(txn_list_with_proof, response)
            .await;
        let latest_version = self.executor_proxy.get_latest_version().await?;
        if latest_version <= previous_version {
            self.peer_manager
                .update_score(peer_id, PeerScoreUpdateType::InvalidChunk);
        } else {
            self.commit(latest_version).await;
        }

        debug!(
            "[state sync] applied chunk. Previous version: {}, new version: {}, chunk size: {}",
            previous_version, self.known_version, chunk_size
        );

        result
    }

    async fn validate_and_store_chunk(
        &mut self,
        txn_list_with_proof: TransactionListWithProof,
        mut response: GetChunkResponse,
    ) -> Result<()> {
        // optimistically fetch next chunk
        let chunk_size = txn_list_with_proof.get_transactions().len() as u64;
        self.request_next_chunk(chunk_size).await;
        debug!(
            "[state sync] process chunk response. chunk_size: {}",
            chunk_size
        );

        let target = LedgerInfo::from_proto(response.take_ledger_info_with_sigs())?;
        self.executor_proxy.validate_ledger_info(&target)?;

        self.store_transactions(target, txn_list_with_proof).await?;

        counters::STATE_SYNC_TXN_REPLAYED.inc_by(chunk_size as i64);

        Ok(())
    }

    /// ensures that StateSynchronizer makes progress
    /// if peer is not responding, issues new sync request
    async fn check_progress(&mut self) {
        if !self.peer_manager.is_empty() && (self.autosync || self.target.is_some()) {
            let last_commit = self.last_commit.unwrap_or(UNIX_EPOCH);
            let timeout = match self.target {
                Some(_) => 2 * self.config.tick_interval_ms,
                None => self.config.tick_interval_ms + self.config.long_poll_timeout_ms,
            };
            let expected_next_sync = last_commit.checked_add(Duration::from_millis(timeout));

            // if coordinator didn't make progress by expected time, issue new request
            if let Some(tst) = expected_next_sync {
                if SystemTime::now().duration_since(tst).is_ok() {
                    self.peer_manager
                        .process_timeout(self.known_version + 1, timeout);
                    self.request_next_chunk(0).await;
                }
            }
        }
    }

    async fn request_next_chunk(&mut self, offset: u64) {
        if self.autosync || self.known_version + offset < self.target_version() {
            if let Some((peer_id, mut sender)) = self.peer_manager.pick_peer() {
                let mut req = GetChunkRequest::new();
                req.set_known_version(self.known_version + offset);
                req.set_limit(self.config.chunk_limit);
                self.peer_manager
                    .process_request(self.known_version + offset + 1, peer_id);
                let timeout = match &self.target {
                    Some(target) => {
                        req.set_ledger_info_with_sigs(target.clone().into_proto());
                        0
                    }
                    None => {
                        req.set_timeout(self.config.long_poll_timeout_ms);
                        self.config.long_poll_timeout_ms
                    }
                };
                debug!(
                    "[state sync] request next chunk. peer_id: {:?}, known_version: {}, timeout: {}",
                    peer_id,
                    self.known_version + offset,
                    timeout
                );

                let mut msg = StateSynchronizerMsg::new();
                msg.set_chunk_request(req);

                if sender.send_to(peer_id, msg).await.is_err() {
                    error!("[state sync] failed to send p2p message");
                }
            }
        }
    }

    async fn store_transactions(
        &self,
        ledger_info: LedgerInfoWithSignatures<Ed25519Signature>,
        txn_list_with_proof: TransactionListWithProof,
    ) -> Result<ExecuteChunkResponse> {
        let mut req = ExecuteChunkRequest::new();
        req.set_txn_list_with_proof(txn_list_with_proof);
        req.set_ledger_info_with_sigs(ledger_info.into_proto());
        self.executor_proxy.execute_chunk(req).await
    }

    async fn check_subscriptions(&mut self) -> Result<()> {
        let ledger_info = self.executor_proxy.get_latest_ledger_info().await?;
        let committed_version = self.known_version;
        let mut ready = vec![];

        self.subscriptions
            .retain(|peer_id, (expiry, known_version, limit)| {
                // filter out expired peer requests
                if SystemTime::now().duration_since(expiry.clone()).is_ok() {
                    return false;
                }
                if *known_version < committed_version {
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
                    ledger_info.clone(),
                    sender,
                ));
            }
        }
        while let Some(res) = futures.next().await {
            if let Err(err) = res {
                error!("[state sync] failed to notify subscriber {:?}", err);
            }
        }
        Ok(())
    }
}
