// Copyright (c) The Libra Core Contributors
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
    counters,
    peer::{Peer, PeerHandle, PeerNotification},
    peer_manager::ConnectionNotification,
    protocols::{
        direct_send::{DirectSend, DirectSendNotification, DirectSendRequest, Message},
        identity::Identity,
        rpc::{InboundRpcRequest, OutboundRpcRequest, Rpc, RpcNotification},
    },
    validator_network, ProtocolId,
};
use channel::{self, libra_channel, message_queues::QueueStyle};
use futures::{
    io::{AsyncRead, AsyncWrite},
    stream::StreamExt,
    FutureExt, SinkExt,
};
use libra_logger::prelude::*;
use libra_types::PeerId;
use netcore::transport::ConnectionOrigin;
use parity_multiaddr::Multiaddr;
use std::{fmt::Debug, marker::PhantomData, num::NonZeroUsize, time::Duration};
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

pub struct NetworkProvider<TSocket>
where
    TSocket: AsyncRead + AsyncWrite + Send + Debug + Unpin + Sync + 'static,
{
    /// Pin the muxer type corresponding to this NetworkProvider instance
    phantom_socket: PhantomData<TSocket>,
}

impl<TSocket> NetworkProvider<TSocket>
where
    TSocket: AsyncRead + AsyncWrite + Send + Debug + Unpin + Sync + 'static,
{
    pub fn start(
        executor: Handle,
        identity: Identity,
        address: Multiaddr,
        origin: ConnectionOrigin,
        connection: TSocket,
        connection_notifs_tx: channel::Sender<ConnectionNotification<TSocket>>,
        max_concurrent_reqs: usize,
        max_concurrent_notifs: usize,
        channel_size: usize,
    ) -> (
        libra_channel::Sender<ProtocolId, NetworkRequest>,
        libra_channel::Receiver<ProtocolId, NetworkNotification>,
    ) {
        let peer_id = identity.peer_id();

        // Setup and start Peer actor.
        let (peer_reqs_tx, peer_reqs_rx) = channel::new(
            channel_size,
            &counters::OP_COUNTERS
                .peer_gauge(&counters::PENDING_PEER_REQUESTS, &peer_id.short_str()),
        );
        let (peer_rpc_notifs_tx, peer_rpc_notifs_rx) = channel::new(
            channel_size,
            &counters::OP_COUNTERS.peer_gauge(
                &counters::PENDING_PEER_RPC_NOTIFICATIONS,
                &peer_id.short_str(),
            ),
        );
        let (peer_ds_notifs_tx, peer_ds_notifs_rx) = channel::new(
            channel_size,
            &counters::OP_COUNTERS.peer_gauge(
                &counters::PENDING_PEER_DIRECT_SEND_NOTIFICATIONS,
                &peer_id.short_str(),
            ),
        );
        let (peer_notifs_tx, peer_notifs_rx) = channel::new(
            channel_size,
            &counters::OP_COUNTERS.peer_gauge(
                &counters::PENDING_PEER_NETWORK_NOTIFICATIONS,
                &peer_id.short_str(),
            ),
        );
        let peer = Peer::new(
            executor.clone(),
            identity,
            address.clone(),
            origin,
            connection,
            peer_reqs_rx,
            peer_notifs_tx,
            peer_rpc_notifs_tx,
            peer_ds_notifs_tx,
        );
        let peer_handle = PeerHandle::new(peer_id, address, peer_reqs_tx);
        executor.spawn(peer.start());

        // Setup and start RPC actor.
        let (rpc_notifs_tx, rpc_notifs_rx) = channel::new(
            channel_size,
            &counters::OP_COUNTERS
                .peer_gauge(&counters::PENDING_RPC_NOTIFICATIONS, &peer_id.short_str()),
        );
        let (rpc_reqs_tx, rpc_reqs_rx) = channel::new(
            channel_size,
            &counters::OP_COUNTERS
                .peer_gauge(&counters::PENDING_RPC_REQUESTS, &peer_id.short_str()),
        );
        let rpc = Rpc::new(
            peer_handle.clone(),
            rpc_reqs_rx,
            peer_rpc_notifs_rx,
            rpc_notifs_tx,
            Duration::from_millis(validator_network::network_builder::INBOUND_RPC_TIMEOUT_MS),
            validator_network::network_builder::MAX_CONCURRENT_OUTBOUND_RPCS,
            validator_network::network_builder::MAX_CONCURRENT_INBOUND_RPCS,
        );
        executor.spawn(rpc.start());

        // Setup and start DirectSend actor.
        let (ds_notifs_tx, ds_notifs_rx) = channel::new(
            channel_size,
            &counters::OP_COUNTERS.peer_gauge(
                &counters::PENDING_DIRECT_SEND_NOTIFICATIONS,
                &peer_id.short_str(),
            ),
        );
        let (ds_reqs_tx, ds_reqs_rx) = channel::new(
            channel_size,
            &counters::OP_COUNTERS.peer_gauge(
                &counters::PENDING_DIRECT_SEND_REQUESTS,
                &peer_id.short_str(),
            ),
        );
        let ds = DirectSend::new(
            peer_handle.clone(),
            ds_reqs_rx,
            ds_notifs_tx,
            peer_ds_notifs_rx,
        );
        executor.spawn(ds.start());

        // TODO: Add label for peer.
        let (requests_tx, requests_rx) = libra_channel::new(
            QueueStyle::FIFO,
            NonZeroUsize::new(channel_size).expect("libra_channel cannot be of size 0"),
            Some(&counters::PENDING_NETWORK_REQUESTS),
        );
        // TODO: Add label for peer.
        let (notifs_tx, notifs_rx) = libra_channel::new(
            QueueStyle::FIFO,
            NonZeroUsize::new(channel_size).expect("libra_channel cannot be of size 0"),
            Some(&counters::PENDING_NETWORK_NOTIFICATIONS),
        );

        // Handle notifications from RPC actor.
        let inbound_rpc_notifs_tx = notifs_tx.clone();
        executor.spawn(rpc_notifs_rx.for_each(move |notif| {
            Self::handle_rpc_notification(peer_id, notif, inbound_rpc_notifs_tx.clone());
            futures::future::ready(())
        }));

        // Handle notifications from DirectSend actor.
        let inbound_ds_notifs_tx = notifs_tx;
        executor.spawn(ds_notifs_rx.for_each(move |notif| {
            Self::handle_ds_notification(peer_id, notif, inbound_ds_notifs_tx.clone());
            futures::future::ready(())
        }));

        // Handle notifications from Peer actor.
        let connection_notifs_tx = connection_notifs_tx;
        executor.spawn(
            peer_notifs_rx.for_each_concurrent(max_concurrent_notifs, move |notif| {
                Self::handle_peer_notification(notif, connection_notifs_tx.clone())
            }),
        );

        // Handle network requests.
        let f = async move {
            let peer_id_str = peer_id.short_str();
            requests_rx
                .for_each_concurrent(max_concurrent_reqs, move |req| {
                    Self::handle_network_request(
                        peer_id,
                        req,
                        rpc_reqs_tx.clone(),
                        ds_reqs_tx.clone(),
                    )
                })
                .then(|_| async move {
                    info!(
                        "Network provider actor terminating for peer: {}",
                        peer_id_str
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
        mut ds_reqs_tx: channel::Sender<DirectSendRequest>,
    ) {
        match req {
            NetworkRequest::SendRpc(req) => {
                if let Err(e) = rpc_reqs_tx.send(req).await {
                    error!(
                        "Failed to send RPC to peer: {}. Error: {:?}",
                        peer_id.short_str(),
                        e
                    );
                }
            }
            NetworkRequest::SendMessage(msg) => {
                counters::LIBRA_NETWORK_DIRECT_SEND_MESSAGES
                    .with_label_values(&["sent"])
                    .inc();
                counters::LIBRA_NETWORK_DIRECT_SEND_BYTES
                    .with_label_values(&["sent"])
                    .observe(msg.mdata.len() as f64);
                if let Err(e) = ds_reqs_tx.send(DirectSendRequest::SendMessage(msg)).await {
                    error!(
                        "Failed to send DirectSend to peer: {}. Error: {:?}",
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
        mut notifs_tx: libra_channel::Sender<ProtocolId, NetworkNotification>,
    ) {
        trace!("RpcNotification::{:?}", notif);
        match notif {
            RpcNotification::RecvRpc(req) => {
                if let Err(e) =
                    notifs_tx.push(req.protocol.clone(), NetworkNotification::RecvRpc(req))
                {
                    warn!("Failed to push RpcNotification to NetworkProvider for peer: {}. Error: {:?}", peer_id.short_str(), e);
                }
            }
        }
    }

    fn handle_ds_notification(
        peer_id: PeerId,
        notif: DirectSendNotification,
        mut notifs_tx: libra_channel::Sender<ProtocolId, NetworkNotification>,
    ) {
        trace!("DirectSendNotification::{:?}", notif);
        match notif {
            DirectSendNotification::RecvMessage(msg) => {
                if let Err(e) =
                    notifs_tx.push(msg.protocol.clone(), NetworkNotification::RecvMessage(msg))
                {
                    warn!("Failed to push DirectSendNotification to NetworkProvider for peer: {}. Error: {:?}", peer_id.short_str(), e);
                }
            }
        }
    }

    async fn handle_peer_notification(
        notif: PeerNotification,
        mut connection_notifs_tx: channel::Sender<ConnectionNotification<TSocket>>,
    ) {
        match notif {
            PeerNotification::PeerDisconnected(identity, addr, origin, reason) => {
                // Send notification to PeerManager. PeerManager is responsible for initiating
                // cleanup.
                if let Err(err) = connection_notifs_tx
                    .send(ConnectionNotification::Disconnected(
                        identity, addr, origin, reason,
                    ))
                    .await
                {
                    warn!("Failed to push Disconnected event to connection event handler. Probably in shutdown mode. Error: {:?}", err);
                }
            }
            _ => {
                unreachable!("Unexpected notification received from Peer actor");
            }
        }
    }
}
