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
    peer_manager::PeerManagerNotification,
    protocols::{
        direct_send::{DirectSendNotification, DirectSendRequest, Message},
        rpc::{InboundRpcRequest, OutboundRpcRequest, RpcNotification, RpcRequest},
    },
    ProtocolId,
};
use channel;
use futures::{FutureExt, SinkExt, StreamExt};
use logger::prelude::*;
use std::{collections::HashMap, fmt::Debug};
use types::PeerId;

/// Requests [`NetworkProvider`] receives from the network interface.
#[derive(Debug)]
pub enum NetworkRequest {
    /// Send an RPC request to a remote peer.
    SendRpc(PeerId, OutboundRpcRequest),
    /// Fire-and-forget style message send to a remote peer.
    SendMessage(PeerId, Message),
    /// Update set of nodes eligible to join the network.
    UpdateEligibleNodes(HashMap<PeerId, NetworkPublicKeys>),
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

pub struct NetworkProvider<TSubstream> {
    /// Map from protocol to upstream handlers for events of that protocol type.
    upstream_handlers: HashMap<ProtocolId, channel::Sender<NetworkNotification>>,
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
    conn_mgr_reqs_tx: channel::Sender<ConnectivityRequest>,
    /// Channel to receive requests from other actors.
    requests_rx: channel::Receiver<NetworkRequest>,
    /// The maximum number of concurrent NetworkRequests that can be handled.
    /// Back-pressure takes effect via bounded mpsc channel beyond the limit.
    max_concurrent_reqs: u32,
    /// The maximum number of concurrent Notifications from Peer Manager,
    /// RPC and Direct Send that can be handled.
    /// Back-pressure takes effect via bounded mpsc channel beyond the limit.
    max_concurrent_notifs: u32,
}

impl<TSubstream> NetworkProvider<TSubstream>
where
    TSubstream: Debug + Send,
{
    pub fn new(
        peer_mgr_notifs_rx: channel::Receiver<PeerManagerNotification<TSubstream>>,
        rpc_reqs_tx: channel::Sender<RpcRequest>,
        rpc_notifs_rx: channel::Receiver<RpcNotification>,
        ds_reqs_tx: channel::Sender<DirectSendRequest>,
        ds_notifs_rx: channel::Receiver<DirectSendNotification>,
        conn_mgr_reqs_tx: channel::Sender<ConnectivityRequest>,
        requests_rx: channel::Receiver<NetworkRequest>,
        upstream_handlers: HashMap<ProtocolId, channel::Sender<NetworkNotification>>,
        max_concurrent_reqs: u32,
        max_concurrent_notifs: u32,
    ) -> Self {
        Self {
            upstream_handlers,
            peer_mgr_notifs_rx,
            rpc_reqs_tx,
            rpc_notifs_rx,
            ds_reqs_tx,
            ds_notifs_rx,
            conn_mgr_reqs_tx,
            requests_rx,
            max_concurrent_reqs,
            max_concurrent_notifs,
        }
    }

    async fn handle_network_request(
        req: NetworkRequest,
        mut rpc_reqs_tx: channel::Sender<RpcRequest>,
        mut ds_reqs_tx: channel::Sender<DirectSendRequest>,
        mut conn_mgr_reqs_tx: channel::Sender<ConnectivityRequest>,
    ) {
        trace!("NetworkRequest::{:?}", req);
        match req {
            NetworkRequest::SendRpc(peer_id, req) => {
                rpc_reqs_tx
                    .send(RpcRequest::SendRpc(peer_id, req))
                    .await
                    .unwrap();
            }
            NetworkRequest::SendMessage(peer_id, msg) => {
                counters::DIRECT_SEND_MESSAGES_SENT.inc();
                counters::DIRECT_SEND_BYTES_SENT.inc_by(msg.mdata.len() as i64);
                ds_reqs_tx
                    .send(DirectSendRequest::SendMessage(peer_id, msg))
                    .await
                    .unwrap();
            }
            NetworkRequest::UpdateEligibleNodes(nodes) => {
                conn_mgr_reqs_tx
                    .send(ConnectivityRequest::UpdateEligibleNodes(nodes))
                    .await
                    .unwrap();
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
                counters::CONNECTED_PEERS.inc();
                for ch in upstream_handlers.values_mut() {
                    ch.send(NetworkNotification::NewPeer(peer_id))
                        .await
                        .unwrap();
                }
            }
            PeerManagerNotification::LostPeer(peer_id, _addr) => {
                counters::CONNECTED_PEERS.dec();
                for ch in upstream_handlers.values_mut() {
                    ch.send(NetworkNotification::LostPeer(peer_id))
                        .await
                        .unwrap();
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
                    ch.send(NetworkNotification::RecvRpc(peer_id, req))
                        .await
                        .unwrap();
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
                counters::DIRECT_SEND_MESSAGES_RECEIVED.inc();
                counters::DIRECT_SEND_BYTES_RECEIVED.inc_by(msg.mdata.len() as i64);
                let ch = upstream_handlers
                    .get_mut(&msg.protocol)
                    .expect("DirectSend protocol not registered");
                ch.send(NetworkNotification::RecvMessage(peer_id, msg))
                    .await
                    .unwrap();
            }
        }
    }

    pub async fn start(self) {
        let rpc_reqs_tx = self.rpc_reqs_tx.clone();
        let ds_reqs_tx = self.ds_reqs_tx.clone();
        let conn_mgr_reqs_tx = self.conn_mgr_reqs_tx.clone();
        let mut reqs = self
            .requests_rx
            .map(move |req| {
                Self::handle_network_request(
                    req,
                    rpc_reqs_tx.clone(),
                    ds_reqs_tx.clone(),
                    conn_mgr_reqs_tx.clone(),
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
    }
}
