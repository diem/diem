// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{counters, executor_proxy::ExecutorProxyTrait, LedgerInfo, PeerId};
use config::config::{NodeConfig, RoleType, StateSyncConfig};
use crypto::ed25519::*;
use execution_proto::proto::execution::{ExecuteChunkRequest, ExecuteChunkResponse};
use failure::prelude::*;
use futures::{
    channel::{mpsc, oneshot},
    compat::Stream01CompatExt,
    stream::{futures_unordered::FuturesUnordered, Fuse},
    StreamExt,
};
use logger::prelude::*;
use network::{
    proto::{GetChunkRequest, GetChunkResponse, StateSynchronizerMsg},
    validator_network::{Event, StateSynchronizerEvents, StateSynchronizerSender},
};
use proto_conv::{FromProto, IntoProto};
use rand::{thread_rng, Rng};
use std::{
    collections::HashMap,
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
    // used for interaction with remote peers
    network_sender: StateSynchronizerSender,
    // used for receiving events from peers
    network_events: Fuse<StateSynchronizerEvents>,
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
    peers: HashMap<PeerId, ()>,
    // option callback. Called when state sync reaches target version
    callback: Option<oneshot::Sender<bool>>,
    // timestamp of next peer sync request
    next_sync: Option<SystemTime>,
    // queue of incoming long polling requests
    // peer will be notified about new chunk of transactions if it's available before expiry time
    // value format is (expiration_time, known_version, limit)
    subscriptions: HashMap<PeerId, (SystemTime, u64, u64)>,
    executor_proxy: T,
}

impl<T: ExecutorProxyTrait> SyncCoordinator<T> {
    pub fn new(
        network_sender: StateSynchronizerSender,
        network_events: StateSynchronizerEvents,
        client_events: mpsc::UnboundedReceiver<CoordinatorMessage>,
        node_config: &NodeConfig,
        executor_proxy: T,
    ) -> Self {
        Self {
            network_sender,
            network_events: network_events.fuse(),
            client_events,

            known_version: 0,
            target: None,
            config: node_config.state_sync.clone(),
            autosync: node_config.base.get_role() == RoleType::FullNode,
            peers: HashMap::new(),
            subscriptions: HashMap::new(),
            callback: None,
            next_sync: None,
            executor_proxy,
        }
    }

    /// main routine. starts sync coordinator that listens for CoordinatorMsg
    pub async fn start(mut self) {
        self.known_version = self
            .executor_proxy
            .get_latest_version()
            .await
            .expect("[start sync] failed to fetch latest version from storage");

        let mut interval =
            Interval::new_interval(Duration::from_millis(self.config.tick_interval_ms))
                .compat()
                .fuse();

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
                network_event = self.network_events.select_next_some() => {
                    match network_event {
                        Ok(event) => {
                            match event {
                                Event::NewPeer(peer_id) => {
                                    if self.autosync {
                                        self.peers.insert(peer_id, ());
                                        self.check_progress().await;
                                    }
                                }
                                Event::LostPeer(peer_id) => {
                                    self.peers.remove(&peer_id);
                                }
                                Event::Message((peer_id, mut message)) => {
                                    if message.has_chunk_request() {
                                        if let Err(err) = self.process_chunk_request(peer_id, message.take_chunk_request()).await {
                                            error!("[state sync] failed to serve chunk request: {:?}", err);
                                        }
                                    }
                                    if message.has_chunk_response() {
                                        self.process_chunk_response(message.take_chunk_response()).await;
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

        // if requested version equals to current committed, just pass ledger info to executor
        // there might be still empty blocks between committed state and requested
        if requested_version <= self.known_version {
            self.store_transactions(target.clone(), TransactionListWithProof::new())
                .await
                .expect("[state sync] failed to execute empty blocks");
            if callback.send(true).is_err() {
                error!("[state sync] coordinator failed to notify subscriber");
            }
            return;
        }

        if requested_version > self.target_version() {
            self.peers = target.signatures().keys().map(|p| (*p, ())).collect();
            self.target = Some(target);
            self.request_next_chunk(0).await;
        }
        self.callback = Some(callback);
    }

    async fn commit(&mut self, version: u64) {
        if version <= self.known_version {
            error!(
                "[state sync] invalid commit. Known version: {}, commit version: {}",
                self.known_version, version
            );
            return;
        }
        self.known_version = version;
        if let Err(err) = self.check_subscriptions().await {
            error!("[state sync] failed to check subscriptions: {:?}", err);
        }
        if self.known_version == self.target_version() {
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
            .unwrap_or(latest_ledger_info);

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
            self.deliver_chunk(
                peer_id,
                request.known_version,
                request.limit,
                target,
                self.network_sender.clone(),
            )
            .await
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
    async fn process_chunk_response(&mut self, mut response: GetChunkResponse) {
        let txn_list_with_proof = response.take_txn_list_with_proof();
        // optimistically fetch next chunk
        let chunk_size = txn_list_with_proof.get_transactions().len() as u64;
        self.request_next_chunk(chunk_size).await;

        match LedgerInfoWithSignatures::<Ed25519Signature>::from_proto(
            response.take_ledger_info_with_sigs(),
        ) {
            Ok(target) => {
                if let Err(err) = self.store_transactions(target, txn_list_with_proof).await {
                    error!("[state sync] failed to apply chunk {:?}", err);
                }
                counters::STATE_SYNC_TXN_REPLAYED.inc_by(chunk_size as i64);
                match self.executor_proxy.get_latest_version().await {
                    Ok(version) => {
                        self.commit(version).await;
                    }
                    Err(err) => {
                        error!("[state sync] storage version read failed {:?}", err);
                    }
                }
            }
            Err(err) => {
                error!("[state sync] invalid ledger info {:?}", err);
            }
        }
    }

    /// ensures that StateSynchronizer makes progress
    /// if peer is not responding, issues new sync request
    async fn check_progress(&mut self) {
        if !self.peers.is_empty() && (self.autosync || self.target.is_some()) {
            let timestamp = self.next_sync.unwrap_or(UNIX_EPOCH);
            if SystemTime::now().duration_since(timestamp).is_ok() {
                self.request_next_chunk(0).await;
            }
        }
    }

    async fn request_next_chunk(&mut self, offset: u64) {
        if self.autosync || self.known_version + offset < self.target_version() {
            let idx = thread_rng().gen_range(0, self.peers.len());
            let peer_id = self
                .peers
                .keys()
                .nth(idx)
                .cloned()
                .expect("[state synchronizer] failed to pick peer from ledger info");

            let mut req = GetChunkRequest::new();
            req.set_known_version(self.known_version + offset);
            req.set_limit(self.config.chunk_limit);
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

            let mut msg = StateSynchronizerMsg::new();
            msg.set_chunk_request(req);

            self.next_sync = SystemTime::now().checked_add(Duration::from_millis(
                timeout + self.config.tick_interval_ms,
            ));
            if self.network_sender.send_to(peer_id, msg).await.is_err() {
                error!("[state sync] failed to send p2p message");
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

        let mut futures: FuturesUnordered<_> = ready
            .into_iter()
            .map(|(peer_id, known_version, limit)| {
                self.deliver_chunk(
                    peer_id,
                    known_version,
                    limit,
                    ledger_info.clone(),
                    self.network_sender.clone(),
                )
            })
            .collect();
        while let Some(res) = futures.next().await {
            if let Err(err) = res {
                error!("[state sync] failed to notify subscriber {:?}", err);
            }
        }
        Ok(())
    }
}
