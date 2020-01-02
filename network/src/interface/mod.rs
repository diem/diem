// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Module exposing generic network API
//!
//! Unlike the [`validator_network`](crate::validator_network) module, which exposes async function
//! call and Stream API specific for consensus and mempool modules, the `interface` module
//! exposes generic network API by receiving requests over a channel for outbound requests and
//! sending notifications to upstream clients for inbound requests and other events. For example,
//! clients wishing to send an RPC need to send a
//! [`NetworkRequest::SendRpc`](crate::interface::NetworkRequest::SendRpc) message to the
//! [`NetworkProvider`] actor. Inbound RPC requests are forwarded to the appropriate
//! handler, determined using the protocol negotiated on the RPC substream.
use crate::{
    common::NetworkPublicKeys,
    connectivity_manager::ConnectivityRequest,
    counters,
    peer_manager::{PeerManagerNotification, PeerManagerRequest},
    protocols::{
        direct_send::{DirectSendNotification, DirectSendRequest, Message},
        rpc::{InboundRpcRequest, OutboundRpcRequest, RpcNotification, RpcRequest},
    },
    validator_network::{
        AdmissionControlNetworkEvents, AdmissionControlNetworkSender, ChainStateNetworkEvents,
        ChainStateNetworkSender, ConsensusNetworkEvents, ConsensusNetworkSender,
        DiscoveryNetworkEvents, DiscoveryNetworkSender, HealthCheckerNetworkEvents,
        HealthCheckerNetworkSender, MempoolNetworkEvents, MempoolNetworkSender,
        StateSynchronizerEvents, StateSynchronizerSender,
    },
    ProtocolId,
};
use channel;
use futures::{channel::oneshot, future::BoxFuture, FutureExt, SinkExt, StreamExt};
use libra_logger::prelude::*;
use libra_types::PeerId;
use parity_multiaddr::Multiaddr;
use std::{collections::HashMap, fmt::Debug, time::Duration};

pub use crate::peer_manager::PeerManagerError;
use futures::lock::Mutex;
use std::collections::HashSet;
use std::sync::Arc;

pub const CONSENSUS_INBOUND_MSG_TIMEOUT_MS: u64 = 10 * 1000; // 10 seconds
pub const MEMPOOL_INBOUND_MSG_TIMEOUT_MS: u64 = 10 * 1000; // 10 seconds
pub const STATE_SYNCHRONIZER_INBOUND_MSG_TIMEOUT_MS: u64 = 10 * 1000; // 10 seconds
pub const ADMISSION_CONTROL_INBOUND_MSG_TIMEOUT_MS: u64 = 10 * 1000; // 10 seconds
pub const HEALTH_CHECKER_INBOUND_MSG_TIMEOUT_MS: u64 = 10 * 1000; // 10 seconds
pub const DISCOVERY_INBOUND_MSG_TIMEOUT_MS: u64 = 10 * 1000; // 10 seconds.
pub const CHAIN_STATE_INBOUND_MSG_TIMEOUT_MS: u64 = 10 * 1000; // 10 seconds.

/// Requests [`NetworkProvider`] receives from the network interface.
#[derive(Debug)]
pub enum NetworkRequest {
    /// Send an RPC request to a remote peer.
    SendRpc(PeerId, OutboundRpcRequest),
    /// Fire-and-forget style message send to a remote peer.
    SendMessage(PeerId, Message),
    /// Update set of nodes eligible to join the network.
    UpdateEligibleNodes(HashMap<PeerId, NetworkPublicKeys>),
    /// Dial the peer at the given `Multiaddr` to establish a connection. When
    /// the dial attempt succeeds or fails, the result will be returned over the
    /// oneshot channel.
    DialPeer(
        PeerId,
        Multiaddr,
        oneshot::Sender<Result<(), PeerManagerError>>,
    ),
    /// Disconnect from the peer with `PeerId`. The disconnect request could fail
    /// if, for example, we are not currently connected with the peer. Results
    /// are sent back to the caller via the oneshot channel.
    DisconnectPeer(PeerId, oneshot::Sender<Result<(), PeerManagerError>>),

    BroadCastMessage(Message, Vec<PeerId>),
}

/// Notifications that [`NetworkProvider`] sends to consumers of its API. The
/// [`NetworkProvider`] in turn receives these notifications from the PeerManager and other
/// [`protocols`](crate::protocols).
#[derive(Debug)]
pub enum NetworkNotification {
    /// Connection with a new peer has been established.
    NewPeer(PeerId),
    /// Connection to a peer has been terminated. This could have been triggered from either end.
    LostPeer(PeerId),
    /// A new RPC request has been received from a remote peer.
    RecvRpc(PeerId, InboundRpcRequest),
    /// A new message has been received from a remote peer.
    RecvMessage(PeerId, Message),
}

/// Trait that any provider of network interface needs to implement.
pub trait LibraNetworkProvider {
    fn add_mempool(
        &mut self,
        mempool_protocols: Vec<ProtocolId>,
    ) -> (MempoolNetworkSender, MempoolNetworkEvents);
    fn add_consensus(
        &mut self,
        consensus_protocols: Vec<ProtocolId>,
    ) -> (ConsensusNetworkSender, ConsensusNetworkEvents);
    fn add_state_synchronizer(
        &mut self,
        state_sync_protocols: Vec<ProtocolId>,
    ) -> (StateSynchronizerSender, StateSynchronizerEvents);
    fn add_admission_control(
        &mut self,
        ac_protocols: Vec<ProtocolId>,
    ) -> (AdmissionControlNetworkSender, AdmissionControlNetworkEvents);
    fn add_health_checker(
        &mut self,
        hc_protocols: Vec<ProtocolId>,
    ) -> (HealthCheckerNetworkSender, HealthCheckerNetworkEvents);
    fn add_discovery(
        &mut self,
        discovery_protocols: Vec<ProtocolId>,
    ) -> (DiscoveryNetworkSender, DiscoveryNetworkEvents);
    fn add_chain_state(
        &mut self,
        chain_state_protocols: Vec<ProtocolId>,
    ) -> (ChainStateNetworkSender, ChainStateNetworkEvents);
    fn start(self: Box<Self>) -> BoxFuture<'static, ()>;
}

pub struct NetworkProvider<TSubstream> {
    /// Map from protocol to upstream handlers for events of that protocol type.
    upstream_handlers: HashMap<ProtocolId, channel::Sender<NetworkNotification>>,
    /// Channel to send requests to the PeerManager actor.
    peer_mgr_reqs_tx: channel::Sender<PeerManagerRequest<TSubstream>>,
    /// Channel over which we receive notifications from PeerManager.
    peer_mgr_notifs_rx: channel::Receiver<PeerManagerNotification<TSubstream>>,
    /// Channel over which we send requets to RPC actor.
    rpc_reqs_tx: channel::Sender<RpcRequest>,
    /// Channel over which we receive notifications from RPC actor.
    rpc_notifs_rx: channel::Receiver<RpcNotification>,
    /// Channel over which we send requests to DirectSend actor.
    ds_reqs_tx: channel::Sender<DirectSendRequest>,
    /// Channel over which we receive notifications from DirectSend actor.
    ds_notifs_rx: channel::Receiver<DirectSendNotification>,
    /// Channel over which we send requests to the ConnectivityManager actor.
    conn_mgr_reqs_tx: Option<channel::Sender<ConnectivityRequest>>,
    /// Channel to receive requests from other actors.
    requests_rx: channel::Receiver<NetworkRequest>,
    /// Channel over which other actors send requests to network.
    requests_tx: channel::Sender<NetworkRequest>,
    /// The maximum number of concurrent NetworkRequests that can be handled.
    /// Back-pressure takes effect via bounded mpsc channel beyond the limit.
    max_concurrent_reqs: u32,
    /// The maximum number of concurrent Notifications from Peer Manager,
    /// RPC and Direct Send that can be handled.
    /// Back-pressure takes effect via bounded mpsc channel beyond the limit.
    max_concurrent_notifs: u32,
    /// Size of channels between different actors.
    channel_size: usize,
    peer_ids: Arc<Mutex<HashSet<PeerId>>>,
}

impl<TSubstream> LibraNetworkProvider for NetworkProvider<TSubstream>
where
    TSubstream: Debug + Send + 'static,
{
    fn add_mempool(
        &mut self,
        mempool_protocols: Vec<ProtocolId>,
    ) -> (MempoolNetworkSender, MempoolNetworkEvents) {
        // Construct Mempool network interfaces
        let (mempool_tx, mempool_rx) = channel::new_with_timeout(
            self.channel_size,
            &counters::PENDING_MEMPOOL_NETWORK_EVENTS,
            Duration::from_millis(MEMPOOL_INBOUND_MSG_TIMEOUT_MS),
        );
        let mempool_network_sender = MempoolNetworkSender::new(self.requests_tx.clone());
        let mempool_network_events = MempoolNetworkEvents::new(mempool_rx);
        let mempool_handlers = mempool_protocols
            .iter()
            .map(|p| (p.clone(), mempool_tx.clone()));
        self.upstream_handlers.extend(mempool_handlers);
        (mempool_network_sender, mempool_network_events)
    }

    fn add_consensus(
        &mut self,
        consensus_protocols: Vec<ProtocolId>,
    ) -> (ConsensusNetworkSender, ConsensusNetworkEvents) {
        // Construct Consensus network interfaces
        let (consensus_tx, consensus_rx) = channel::new_with_timeout(
            self.channel_size,
            &counters::PENDING_CONSENSUS_NETWORK_EVENTS,
            Duration::from_millis(CONSENSUS_INBOUND_MSG_TIMEOUT_MS),
        );
        let consensus_network_sender = ConsensusNetworkSender::new(self.requests_tx.clone());
        let consensus_network_events = ConsensusNetworkEvents::new(consensus_rx);
        let consensus_handlers = consensus_protocols
            .iter()
            .map(|p| (p.clone(), consensus_tx.clone()));
        self.upstream_handlers.extend(consensus_handlers);
        (consensus_network_sender, consensus_network_events)
    }

    fn add_state_synchronizer(
        &mut self,
        state_sync_protocols: Vec<ProtocolId>,
    ) -> (StateSynchronizerSender, StateSynchronizerEvents) {
        // Construct StateSynchronizer network interfaces
        let (state_sync_tx, state_sync_rx) = channel::new_with_timeout(
            self.channel_size,
            &counters::PENDING_STATE_SYNCHRONIZER_NETWORK_EVENTS,
            Duration::from_millis(STATE_SYNCHRONIZER_INBOUND_MSG_TIMEOUT_MS),
        );
        let state_sync_network_sender = StateSynchronizerSender::new(self.requests_tx.clone());
        let state_sync_network_events = StateSynchronizerEvents::new(state_sync_rx);
        let state_sync_handlers = state_sync_protocols
            .iter()
            .map(|p| (p.clone(), state_sync_tx.clone()));
        self.upstream_handlers.extend(state_sync_handlers);
        (state_sync_network_sender, state_sync_network_events)
    }

    fn add_admission_control(
        &mut self,
        ac_protocols: Vec<ProtocolId>,
    ) -> (AdmissionControlNetworkSender, AdmissionControlNetworkEvents) {
        // Construct Admission Control network interfaces
        let (ac_tx, ac_rx) = channel::new_with_timeout(
            self.channel_size,
            &counters::PENDING_ADMISSION_CONTROL_NETWORK_EVENTS,
            Duration::from_millis(ADMISSION_CONTROL_INBOUND_MSG_TIMEOUT_MS),
        );
        let ac_network_sender = AdmissionControlNetworkSender::new(self.requests_tx.clone());
        let ac_network_events = AdmissionControlNetworkEvents::new(ac_rx);
        let ac_handlers = ac_protocols.iter().map(|p| (p.clone(), ac_tx.clone()));
        self.upstream_handlers.extend(ac_handlers);
        (ac_network_sender, ac_network_events)
    }

    fn add_health_checker(
        &mut self,
        hc_protocols: Vec<ProtocolId>,
    ) -> (HealthCheckerNetworkSender, HealthCheckerNetworkEvents) {
        // Construct Health Checker network interfaces
        let (hc_tx, hc_rx) = channel::new_with_timeout(
            self.channel_size,
            &counters::PENDING_HEALTH_CHECKER_NETWORK_EVENTS,
            Duration::from_millis(HEALTH_CHECKER_INBOUND_MSG_TIMEOUT_MS),
        );
        let hc_network_sender = HealthCheckerNetworkSender::new(self.requests_tx.clone());
        let hc_network_events = HealthCheckerNetworkEvents::new(hc_rx);
        let hc_handlers = hc_protocols.iter().map(|p| (p.clone(), hc_tx.clone()));
        self.upstream_handlers.extend(hc_handlers);
        (hc_network_sender, hc_network_events)
    }

    fn add_discovery(
        &mut self,
        discovery_protocols: Vec<ProtocolId>,
    ) -> (DiscoveryNetworkSender, DiscoveryNetworkEvents) {
        // Construct Discovery Network interfaces
        let (discovery_tx, discovery_rx) = channel::new_with_timeout(
            self.channel_size,
            &counters::PENDING_DISCOVERY_NETWORK_EVENTS,
            Duration::from_millis(DISCOVERY_INBOUND_MSG_TIMEOUT_MS),
        );
        let discovery_network_sender = DiscoveryNetworkSender::new(self.requests_tx.clone());
        let discovery_network_events = DiscoveryNetworkEvents::new(discovery_rx);
        let discovery_handlers = discovery_protocols
            .iter()
            .map(|p| (p.clone(), discovery_tx.clone()));
        self.upstream_handlers.extend(discovery_handlers);
        (discovery_network_sender, discovery_network_events)
    }

    fn add_chain_state(
        &mut self,
        chain_state_protocols: Vec<ProtocolId>,
    ) -> (ChainStateNetworkSender, ChainStateNetworkEvents) {
        let (chain_state_tx, chain_state_rx) = channel::new_with_timeout(
            self.channel_size,
            &counters::TEST_NETWORK_REQUESTS,
            Duration::from_millis(CHAIN_STATE_INBOUND_MSG_TIMEOUT_MS),
        );
        let chain_state_network_sender = ChainStateNetworkSender::new(self.requests_tx.clone());
        let chain_state_network_events = ChainStateNetworkEvents::new(chain_state_rx);
        let chain_state_handlers = chain_state_protocols
            .iter()
            .map(|p| (p.clone(), chain_state_tx.clone()));
        self.upstream_handlers.extend(chain_state_handlers);
        (chain_state_network_sender, chain_state_network_events)
    }

    fn start(self: Box<Self>) -> BoxFuture<'static, ()> {
        let f = async move {
            let peer_mgr_reqs_tx = self.peer_mgr_reqs_tx.clone();
            let rpc_reqs_tx = self.rpc_reqs_tx.clone();
            let ds_reqs_tx = self.ds_reqs_tx.clone();
            let conn_mgr_reqs_tx = self.conn_mgr_reqs_tx.clone();
            let peer_ids = self.peer_ids.clone();
            let mut reqs = self
                .requests_rx
                .map(move |req| {
                    Self::handle_network_request(
                        req,
                        peer_mgr_reqs_tx.clone(),
                        rpc_reqs_tx.clone(),
                        ds_reqs_tx.clone(),
                        conn_mgr_reqs_tx.clone(),
                        peer_ids.clone(),
                    )
                    .boxed()
                })
                .buffer_unordered(self.max_concurrent_reqs as usize);

            let upstream_handlers = self.upstream_handlers.clone();
            let mut peer_mgr_notifs = self
                .peer_mgr_notifs_rx
                .map(move |notif| {
                    Self::handle_peer_mgr_notification(notif, upstream_handlers.clone()).boxed()
                })
                .buffer_unordered(self.max_concurrent_notifs as usize);

            let upstream_handlers = self.upstream_handlers.clone();
            let mut rpc_notifs = self
                .rpc_notifs_rx
                .map(move |notif| {
                    Self::handle_rpc_notification(notif, upstream_handlers.clone()).boxed()
                })
                .buffer_unordered(self.max_concurrent_notifs as usize);

            let upstream_handlers = self.upstream_handlers.clone();
            let mut ds_notifs = self
                .ds_notifs_rx
                .map(|notif| Self::handle_ds_notification(upstream_handlers.clone(), notif).boxed())
                .buffer_unordered(self.max_concurrent_notifs as usize);

            loop {
                futures::select! {
                    _ = reqs.select_next_some() => {},
                    _ = peer_mgr_notifs.select_next_some() => {},
                    _ = rpc_notifs.select_next_some() => {},
                    _ = ds_notifs.select_next_some() => {}
                    complete => {
                        crit!("Network provider actor terminated");
                        break;
                    }
                }
            }
        };
        f.boxed()
    }
}

impl<TSubstream> NetworkProvider<TSubstream>
where
    TSubstream: Debug + Send,
{
    pub fn new(
        peer_mgr_reqs_tx: channel::Sender<PeerManagerRequest<TSubstream>>,
        peer_mgr_notifs_rx: channel::Receiver<PeerManagerNotification<TSubstream>>,
        rpc_reqs_tx: channel::Sender<RpcRequest>,
        rpc_notifs_rx: channel::Receiver<RpcNotification>,
        ds_reqs_tx: channel::Sender<DirectSendRequest>,
        ds_notifs_rx: channel::Receiver<DirectSendNotification>,
        conn_mgr_reqs_tx: Option<channel::Sender<ConnectivityRequest>>,
        requests_rx: channel::Receiver<NetworkRequest>,
        requests_tx: channel::Sender<NetworkRequest>,
        max_concurrent_reqs: u32,
        max_concurrent_notifs: u32,
        channel_size: usize,
        peer_ids: Arc<Mutex<HashSet<PeerId>>>,
    ) -> Self {
        Self {
            upstream_handlers: HashMap::new(),
            peer_mgr_reqs_tx,
            peer_mgr_notifs_rx,
            rpc_reqs_tx,
            rpc_notifs_rx,
            ds_reqs_tx,
            ds_notifs_rx,
            conn_mgr_reqs_tx,
            requests_rx,
            requests_tx,
            max_concurrent_reqs,
            max_concurrent_notifs,
            channel_size,
            peer_ids,
        }
    }

    async fn handle_network_request(
        req: NetworkRequest,
        mut peer_mgr_reqs_tx: channel::Sender<PeerManagerRequest<TSubstream>>,
        mut rpc_reqs_tx: channel::Sender<RpcRequest>,
        mut ds_reqs_tx: channel::Sender<DirectSendRequest>,
        conn_mgr_reqs_tx: Option<channel::Sender<ConnectivityRequest>>,
        peer_ids: Arc<Mutex<HashSet<PeerId>>>,
    ) {
        trace!("NetworkRequest::{:?}", req);
        match req {
            NetworkRequest::SendRpc(peer_id, req) => {
                match rpc_reqs_tx
                    .send(RpcRequest::SendRpc(peer_id, req))
                    .await {
                    Ok(_) => {},
                    Err(e) => {
                        error!(
                            "handle_network_request: peer :{:?} send rpc err: {:?}",
                            peer_id,
                            e,
                        );
                    }
                };
            }
            NetworkRequest::SendMessage(peer_id, msg) => {
                counters::LIBRA_NETWORK_DIRECT_SEND_MESSAGES
                    .with_label_values(&["sent"])
                    .inc();
                counters::LIBRA_NETWORK_DIRECT_SEND_BYTES
                    .with_label_values(&["sent"])
                    .observe(msg.mdata.len() as f64);
                match ds_reqs_tx
                    .send(DirectSendRequest::SendMessage(peer_id, msg))
                    .await {
                    Ok(_) => {},
                    Err(e) => {
                        error!(
                            "handle_network_request: peer :{:?} send msg err: {:?}",
                            peer_id,
                            e,
                        );
                    }
                };
            }
            NetworkRequest::BroadCastMessage(msg, ignore_peers) => {
                let peer_ids_clone = peer_ids.lock().await;
                for peer_id in peer_ids_clone.iter() {
                    if !ignore_peers.contains(&peer_id) {
                        match ds_reqs_tx
                            .send(DirectSendRequest::SendMessage(peer_id.clone(), msg.clone()))
                            .await {
                            Ok(_) => {},
                            Err(e) => {
                                error!(
                                    "handle_network_request: peer :{:?} broadcast msg err: {:?}",
                                    peer_id,
                                    e,
                                );
                            }
                        };
                    }
                }
            }
            NetworkRequest::UpdateEligibleNodes(nodes) => {
                let mut conn_mgr_reqs_tx = conn_mgr_reqs_tx
                    .expect("Received requst to update eligible nodes in permissionless network");
                match conn_mgr_reqs_tx
                    .send(ConnectivityRequest::UpdateEligibleNodes(nodes))
                    .await {
                    Ok(_) => {},
                    Err(e) => {
                        error!(
                            "handle_network_request: update eligible nodes err: {:?}",
                            e,
                        );
                    }
                };
            }
            NetworkRequest::DialPeer(peer_id, addr, sender) => {
                match peer_mgr_reqs_tx
                    .send(PeerManagerRequest::DialPeer(peer_id, addr, sender))
                    .await {
                    Ok(_) => {},
                    Err(e) => {
                        error!(
                            "handle_network_request: peer :{:?} dial peer err: {:?}",
                            peer_id,
                            e,
                        );
                    }
                };
            }
            NetworkRequest::DisconnectPeer(peer_id, sender) => {
                match peer_mgr_reqs_tx
                    .send(PeerManagerRequest::DisconnectPeer(peer_id, sender))
                    .await {
                    Ok(_) => {},
                    Err(e) => {
                        error!(
                            "handle_network_request: peer :{:?} disconnect peer err: {:?}",
                            peer_id,
                            e,
                        );
                    }
                };
            }
        }
    }

    async fn handle_peer_mgr_notification(
        notif: PeerManagerNotification<TSubstream>,
        mut upstream_handlers: HashMap<ProtocolId, channel::Sender<NetworkNotification>>,
    ) {
        trace!("PeerManagerNotification::{:?}", notif);
        match notif {
            PeerManagerNotification::NewPeer(peer_id, _addr) => {
                for ch in upstream_handlers.values_mut() {
                    match ch.send(NetworkNotification::NewPeer(peer_id))
                        .await {
                        Ok(_) => {},
                        Err(e) => {
                            error!(
                                "handle_peer_mgr_notification: peer :{:?} new peer err: {:?}",
                                peer_id,
                                e,
                            );
                        }
                    };
                }
            }
            PeerManagerNotification::LostPeer(peer_id, _addr) => {
                for ch in upstream_handlers.values_mut() {
                    match ch.send(NetworkNotification::LostPeer(peer_id))
                        .await {
                        Ok(_) => {},
                        Err(e) => {
                            error!(
                                "handle_ds_notification: peer :{:?} lost peer err: {:?}",
                                peer_id,
                                e,
                            );
                        }
                    };
                }
            }
            _ => {
                unreachable!("Received unexpected event from PeerManager");
            }
        }
    }

    async fn handle_rpc_notification(
        notif: RpcNotification,
        mut upstream_handlers: HashMap<ProtocolId, channel::Sender<NetworkNotification>>,
    ) {
        trace!("RpcNotification::{:?}", notif);
        match notif {
            RpcNotification::RecvRpc(peer_id, req) => {
                if let Some(ch) = upstream_handlers.get_mut(&req.protocol) {
                    match ch.send(NetworkNotification::RecvRpc(peer_id, req))
                        .await {
                        Ok(_) => {},
                        Err(e) => {
                            error!(
                                "handle_rpc_notification: peer :{:?} send msg err: {:?}",
                                peer_id,
                                e,
                            );
                        }
                    };
                } else {
                    unreachable!();
                }
            }
        }
    }

    async fn handle_ds_notification(
        mut upstream_handlers: HashMap<ProtocolId, channel::Sender<NetworkNotification>>,
        notif: DirectSendNotification,
    ) {
        trace!("DirectSendNotification::{:?}", notif);
        match notif {
            DirectSendNotification::RecvMessage(peer_id, msg) => {
                counters::LIBRA_NETWORK_DIRECT_SEND_MESSAGES
                    .with_label_values(&["received"])
                    .inc();
                counters::LIBRA_NETWORK_DIRECT_SEND_BYTES
                    .with_label_values(&["received"])
                    .observe(msg.mdata.len() as f64);
                let ch = upstream_handlers
                    .get_mut(&msg.protocol)
                    .expect("DirectSend protocol not registered");
                match ch.send(NetworkNotification::RecvMessage(peer_id, msg))
                    .await {
                    Ok(_) => {},
                    Err(e) => {
                        error!(
                            "handle_ds_notification: peer :{:?} send msg err: {:?}",
                            peer_id,
                            e,
                        );
                    }
                };
            }
        }
    }
}
