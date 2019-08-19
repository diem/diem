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
    stream::Fuse,
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
    collections::{BTreeMap, HashMap},
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::timer::Interval;
use types::{ledger_info::LedgerInfoWithSignatures, proto::transaction::TransactionListWithProof};

/// message used for communication between Consensus and Coordinator
pub enum CoordinatorMessage {
    // used to initiate new sync
    Requested(LedgerInfo, oneshot::Sender<bool>),
    // used to notify about new txn commit
    Commit(u64),
    GetState(oneshot::Sender<u64>),
}

/// used to coordinate synchronization process
/// handles Consensus requests and drives sync with remote peers
pub struct SyncCoordinator<T> {
    // used for interaction with remote peers
    network_sender: StateSynchronizerSender,
    // used for receiving events from peers
    network_events: Fuse<StateSynchronizerEvents>,
    // used to process Consensus events
    consensus_events: mpsc::UnboundedReceiver<CoordinatorMessage>,

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
    // subscribers of synchronization
    // each of them will be notified once their target version is ready
    subscribers: BTreeMap<u64, Vec<oneshot::Sender<bool>>>,
    // timestamp of last peer sync request
    last_sync: Option<SystemTime>,
    executor_proxy: T,
}

impl<T: ExecutorProxyTrait> SyncCoordinator<T> {
    pub fn new(
        network_sender: StateSynchronizerSender,
        network_events: StateSynchronizerEvents,
        consensus_events: mpsc::UnboundedReceiver<CoordinatorMessage>,
        node_config: &NodeConfig,
        executor_proxy: T,
    ) -> Self {
        Self {
            network_sender,
            network_events: network_events.fuse(),
            consensus_events,

            known_version: 0,
            target: None,
            config: node_config.state_sync.clone(),
            autosync: node_config.base.get_role() == RoleType::FullNode,
            peers: HashMap::new(),

            subscribers: BTreeMap::new(),
            last_sync: None,
            executor_proxy,
        }
    }

    /// main routine. starts sync coordinator that listens for CoordinatorMsg
    pub async fn start(mut self) {
        self.known_version = self
            .get_latest_version()
            .await
            .expect("[start sync] failed to fetch latest version from storage");

        let mut interval =
            Interval::new_interval(Duration::from_millis(self.config.tick_interval_ms))
                .compat()
                .fuse();

        loop {
            ::futures::select! {
                msg = self.consensus_events.select_next_some() => {
                    match msg {
                        CoordinatorMessage::Requested(target, subscription) => {
                            self.consensus_sync(target, subscription).await;
                        }
                        CoordinatorMessage::Commit(version) => {
                             self.handle_commit(version);
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
                                    }
                                }
                                Event::LostPeer(peer_id) => {
                                    self.peers.remove(&peer_id);
                                }
                                Event::Message((peer_id, mut message)) => {
                                    if message.has_chunk_request() {
                                        self.process_chunk_request(peer_id, message.take_chunk_request()).await;
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

    async fn get_latest_version(&self) -> Result<u64> {
        self.executor_proxy
            .get_latest_ledger_info()
            .await
            .map(|li| li.ledger_info().version())
    }

    async fn consensus_sync(&mut self, target: LedgerInfo, subscriber: oneshot::Sender<bool>) {
        let requested_version = target.ledger_info().version();
        self.known_version = self
            .get_latest_version()
            .await
            .expect("[state sync] failed to fetch latest version from storage");

        // if requested version equals to current committed, just pass ledger info to executor
        // there might be still empty blocks between committed state and requested
        if requested_version <= self.known_version {
            self.store_transactions(target.clone(), TransactionListWithProof::new())
                .await
                .expect("[state sync] failed to execute empty blocks");
            if subscriber.send(true).is_err() {
                error!("[state sync] coordinator failed to notify subscriber");
            }
            return;
        }

        if requested_version > self.target_version() {
            self.peers = target.signatures().keys().map(|p| (*p, ())).collect();
            self.target = Some(target);
            self.request_next_chunk(0).await;
        }

        self.subscribers
            .entry(requested_version)
            .or_insert_with(|| vec![])
            .push(subscriber);
    }

    fn handle_commit(&mut self, version: u64) {
        self.known_version = std::cmp::max(version, self.known_version);
    }

    fn get_state(&self, callback: oneshot::Sender<u64>) {
        if callback.send(self.known_version).is_err() {
            error!("[state sync] failed to fetch internal state");
        }
    }

    /// Get a batch of transactions
    async fn process_chunk_request(&mut self, peer_id: PeerId, mut request: GetChunkRequest) {
        let latest_ledger_info = match self.executor_proxy.get_latest_ledger_info().await {
            Ok(li) => li,
            Err(err) => {
                error!("[state sync] failed to fetch latest ledger info: {:?}", err);
                return;
            }
        };
        let target = LedgerInfo::from_proto(request.take_ledger_info_with_sigs())
            .unwrap_or(latest_ledger_info);

        match self
            .executor_proxy
            .get_chunk(request.known_version, request.limit, target)
            .await
        {
            Ok(response) => {
                let mut msg = StateSynchronizerMsg::new();
                msg.set_chunk_response(response);
                if self.network_sender.send_to(peer_id, msg).await.is_err() {
                    error!("[state sync] failed to send p2p message");
                }
            }
            Err(err) => {
                error!("[state sync] executor error {:?}", err);
            }
        }
    }

    /// processes batch of transactions downloaded from peer
    /// executes transactions, updates progress state, notifies subscribers if some sync is finished
    async fn process_chunk_response(&mut self, mut response: GetChunkResponse) {
        let txn_list_with_proof = response.take_txn_list_with_proof();
        // optimistically fetch next chunk
        let chunk_size = txn_list_with_proof.get_transactions().len() as u64;
        self.request_next_chunk(chunk_size).await;

        match LedgerInfoWithSignatures::<Ed25519Signature>::from_proto(
            response.take_ledger_info_with_sigs(),
        ) {
            Ok(target) => {
                let status = self.store_transactions(target, txn_list_with_proof).await;
                counters::STATE_SYNC_TXN_REPLAYED.inc_by(chunk_size as i64);
                if status.is_ok() {
                    match self.get_latest_version().await {
                        Ok(version) => {
                            self.known_version = version;
                        }
                        Err(err) => {
                            error!("[state sync] storage version read failed {:?}", err);
                        }
                    }
                }
                self.notify_subscribers(status.is_ok());
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
            let timestamp = self.last_sync.unwrap_or(UNIX_EPOCH);
            let delta = SystemTime::now()
                .duration_since(timestamp)
                .expect("system time failure");
            if delta.as_millis() > (2 * u128::from(self.config.tick_interval_ms)) {
                self.last_sync = None;
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
            if let Some(target) = &self.target {
                req.set_ledger_info_with_sigs(target.clone().into_proto());
            }
            let mut msg = StateSynchronizerMsg::new();
            msg.set_chunk_request(req);

            self.last_sync = Some(SystemTime::now());
            if self.network_sender.send_to(peer_id, msg).await.is_err() {
                error!("[state sync] failed to send p2p message");
            }
        }
    }

    fn notify_subscribers(&mut self, result: bool) {
        let mut active_subscribers = self.subscribers.split_off(&(self.known_version + 1));

        // notify subscribers if some syncs are ready
        for channels in self.subscribers.values_mut() {
            channels.drain(..).for_each(|ch| {
                if ch.send(result).is_err() {
                    error!("[state sync] coordinator failed to notify subscriber");
                }
            });
        }
        self.subscribers.clear();
        self.subscribers.append(&mut active_subscribers);
        // reset sync state if done
        if self.subscribers.is_empty() {
            self.target = None;
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
}
