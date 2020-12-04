// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Module exposing generic network interface to a single connected peer.
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
    constants, counters,
    logging::NetworkSchema,
    peer::{Peer, PeerHandle, PeerNotification, PeerRequest},
    peer_manager::TransportNotification,
    protocols::{
        direct_send::Message,
        rpc::{InboundRpcRequest, OutboundRpcRequest, Rpc, RpcNotification},
    },
    transport::Connection,
    ProtocolId,
};
use channel::{self, diem_channel, message_queues::QueueStyle};
use diem_config::network_id::NetworkContext;
use diem_logger::prelude::*;
use diem_types::PeerId;
use futures::{
    io::{AsyncRead, AsyncWrite},
    stream::StreamExt,
    FutureExt, SinkExt,
};
use std::{fmt::Debug, marker::PhantomData, num::NonZeroUsize, sync::Arc, time::Duration};
use tokio::runtime::Handle;

/// Requests [`NetworkProvider`] receives from the network interface.
#[derive(Debug)]
pub enum NetworkRequest {
    /// Send an RPC request to peer.
    SendRpc(OutboundRpcRequest),
    /// Fire-and-forget style message send to peer.
    SendMessage(Message),
}

/// Notifications that [`NetworkProvider`] sends to consumers of its API. The
/// [`NetworkProvider`] in turn receives these notifications from the PeerManager and other
/// [`protocols`](crate::protocols).
#[derive(Debug)]
pub enum NetworkNotification {
    /// A new RPC request has been received from peer.
    RecvRpc(InboundRpcRequest),
    /// A new message has been received from peer.
    RecvMessage(Message),
}

pub struct NetworkProvider<TSocket> {
    /// Pin the muxer type corresponding to this NetworkProvider instance
    phantom_socket: PhantomData<TSocket>,
}

impl<TSocket> NetworkProvider<TSocket>
where
    TSocket: AsyncRead + AsyncWrite + Send + 'static,
{
    pub fn start(
        network_context: Arc<NetworkContext>,
        executor: Handle,
        connection: Connection<TSocket>,
        connection_notifs_tx: channel::Sender<TransportNotification<TSocket>>,
        max_concurrent_reqs: usize,
        max_concurrent_notifs: usize,
        channel_size: usize,
        max_frame_size: usize,
    ) -> (
        diem_channel::Sender<ProtocolId, NetworkRequest>,
        diem_channel::Receiver<ProtocolId, NetworkNotification>,
    ) {
        let peer_id = connection.metadata.remote_peer_id;

        // Setup and start Peer actor.
        let (peer_reqs_tx, peer_reqs_rx) =
            channel::new(channel_size, &counters::PENDING_PEER_REQUESTS);
        let (peer_rpc_notifs_tx, peer_rpc_notifs_rx) =
            channel::new(channel_size, &counters::PENDING_PEER_RPC_NOTIFICATIONS);
        let (peer_notifs_tx, peer_notifs_rx) =
            channel::new(channel_size, &counters::PENDING_PEER_NETWORK_NOTIFICATIONS);
        let peer_handle = PeerHandle::new(
            network_context.clone(),
            connection.metadata.clone(),
            peer_reqs_tx.clone(),
        );
        let peer = Peer::new(
            Arc::clone(&network_context),
            executor.clone(),
            connection,
            peer_reqs_rx,
            peer_notifs_tx,
            peer_rpc_notifs_tx,
            max_frame_size,
        );
        executor.spawn(peer.start());

        // Setup and start RPC actor.
        let (rpc_notifs_tx, rpc_notifs_rx) =
            channel::new(channel_size, &counters::PENDING_RPC_NOTIFICATIONS);
        let (rpc_reqs_tx, rpc_reqs_rx) =
            channel::new(channel_size, &counters::PENDING_RPC_REQUESTS);
        let rpc = Rpc::new(
            Arc::clone(&network_context),
            peer_handle.clone(),
            rpc_reqs_rx,
            peer_rpc_notifs_rx,
            rpc_notifs_tx,
            Duration::from_millis(constants::INBOUND_RPC_TIMEOUT_MS),
            constants::MAX_CONCURRENT_OUTBOUND_RPCS,
            constants::MAX_CONCURRENT_INBOUND_RPCS,
        );
        executor.spawn(rpc.start());

        // TODO: Add label for peer.
        let (requests_tx, requests_rx) = diem_channel::new(
            QueueStyle::FIFO,
            NonZeroUsize::new(channel_size).expect("diem_channel cannot be of size 0"),
            Some(&counters::PENDING_NETWORK_REQUESTS),
        );
        // TODO: Add label for peer.
        let (notifs_tx, notifs_rx) = diem_channel::new(
            QueueStyle::FIFO,
            NonZeroUsize::new(channel_size).expect("diem_channel cannot be of size 0"),
            Some(&counters::PENDING_NETWORK_NOTIFICATIONS),
        );

        // Handle notifications from RPC actor.
        let inbound_rpc_notifs_tx = notifs_tx.clone();
        executor.spawn(rpc_notifs_rx.for_each(move |notif| {
            Self::handle_rpc_notification(peer_id, notif, inbound_rpc_notifs_tx.clone());
            futures::future::ready(())
        }));

        // Handle notifications from Peer actor.
        let inbound_notifs_tx = notifs_tx;
        let connection_notifs_tx = connection_notifs_tx;
        executor.spawn(
            peer_notifs_rx.for_each_concurrent(max_concurrent_notifs, move |notif| {
                Self::handle_peer_notification(
                    notif,
                    inbound_notifs_tx.clone(),
                    connection_notifs_tx.clone(),
                )
            }),
        );

        // Handle network requests.
        let f = async move {
            requests_rx
                .for_each_concurrent(max_concurrent_reqs, move |req| {
                    Self::handle_network_request(
                        peer_id,
                        req,
                        rpc_reqs_tx.clone(),
                        peer_reqs_tx.clone(),
                    )
                })
                .then(|_| async move {
                    info!(
                        NetworkSchema::new(&network_context).remote_peer(&peer_id),
                        "{} Network peer actor terminating for peer: {}",
                        network_context,
                        peer_id.short_str()
                    );
                    // Cleanly close connection with peer.
                    let mut peer_handle = peer_handle;
                    peer_handle.disconnect().await;
                })
                .await;
        };
        executor.spawn(f);

        (requests_tx, notifs_rx)
    }

    async fn handle_network_request(
        peer_id: PeerId,
        req: NetworkRequest,
        mut rpc_reqs_tx: channel::Sender<OutboundRpcRequest>,
        mut peer_reqs_tx: channel::Sender<PeerRequest>,
    ) {
        match req {
            NetworkRequest::SendRpc(req) => {
                if let Err(e) = rpc_reqs_tx.send(req).await {
                    error!(
                        remote_peer = peer_id,
                        error = %e,
                        "Failed to send RPC to peer: {}. Error: {}",
                        peer_id.short_str(),
                        e
                    );
                }
            }
            NetworkRequest::SendMessage(msg) => {
                if let Err(e) = peer_reqs_tx.send(PeerRequest::SendDirectSend(msg)).await {
                    error!(
                        remote_peer = peer_id,
                        error = %e,
                        "Failed to send DirectSend to peer: {}. Error: {}",
                        peer_id.short_str(),
                        e
                    );
                }
            }
        }
    }

    fn handle_rpc_notification(
        peer_id: PeerId,
        notif: RpcNotification,
        mut notifs_tx: diem_channel::Sender<ProtocolId, NetworkNotification>,
    ) {
        trace!("RpcNotification::{:?}", notif);
        match notif {
            RpcNotification::RecvRpc(req) => {
                if let Err(e) = notifs_tx.push(req.protocol_id, NetworkNotification::RecvRpc(req)) {
                    warn!(
                        remote_peer = peer_id,
                        error = e.to_string(),
                        "Failed to push RpcNotification to NetworkProvider for peer: {}. Error: {:?}",
                        peer_id.short_str(),
                        e
                    );
                }
            }
        }
    }

    async fn handle_peer_notification(
        notif: PeerNotification,
        mut inbound_notifs_tx: diem_channel::Sender<ProtocolId, NetworkNotification>,
        mut connection_notifs_tx: channel::Sender<TransportNotification<TSocket>>,
    ) {
        match notif {
            PeerNotification::PeerDisconnected(conn_info, reason) => {
                // Send notification to PeerManager. PeerManager is responsible for initiating
                // cleanup.
                if let Err(err) = connection_notifs_tx
                    .send(TransportNotification::Disconnected(conn_info, reason))
                    .await
                {
                    warn!(
                        error = err.to_string(),
                        "Failed to push Disconnected event to connection event handler. Probably in shutdown mode. Error: {:?}",
                        err
                    );
                }
            }
            PeerNotification::RecvDirectSend(message) => {
                let protocol_id = message.protocol_id;
                let notif = NetworkNotification::RecvMessage(message);
                if let Err(e) = inbound_notifs_tx.push(protocol_id, notif) {
                    warn!(
                        error = e.to_string(),
                        "Failed to push RecvDirectSend to PeerManager. Error: {:?}", e
                    );
                }
            }
            _ => {
                warn!(
                    "Unexpected notification received from Peer actor: {:?}",
                    notif
                );
            }
        }
    }
}
