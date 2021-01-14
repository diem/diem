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
    peer::Peer,
    peer_manager::TransportNotification,
    protocols::{
        direct_send::Message,
        rpc::{InboundRpcRequest, OutboundRpcRequest},
    },
    transport::Connection,
    ProtocolId,
};
use channel::{self, diem_channel, message_queues::QueueStyle};
use diem_config::network_id::NetworkContext;
use diem_rate_limiter::rate_limit::SharedBucket;
use futures::io::{AsyncRead, AsyncWrite};
use std::{fmt::Debug, sync::Arc, time::Duration};
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
#[derive(Debug, PartialEq)]
pub enum NetworkNotification {
    /// A new RPC request has been received from peer.
    RecvRpc(InboundRpcRequest),
    /// A new message has been received from peer.
    RecvMessage(Message),
}

pub struct NetworkProvider;

impl NetworkProvider {
    pub fn start<TSocket>(
        network_context: Arc<NetworkContext>,
        executor: Handle,
        connection: Connection<TSocket>,
        connection_notifs_tx: channel::Sender<TransportNotification<TSocket>>,
        _max_concurrent_reqs: usize,
        _max_concurrent_notifs: usize,
        channel_size: usize,
        max_frame_size: usize,
        inbound_rate_limiter: Option<SharedBucket>,
        outbound_rate_limiter: Option<SharedBucket>,
    ) -> (
        diem_channel::Sender<ProtocolId, NetworkRequest>,
        diem_channel::Receiver<ProtocolId, NetworkNotification>,
    )
    where
        TSocket: AsyncRead + AsyncWrite + Send + 'static,
    {
        // TODO: Add label for peer.
        let (network_reqs_tx, network_reqs_rx) = diem_channel::new(
            QueueStyle::FIFO,
            channel_size,
            Some(&counters::PENDING_NETWORK_REQUESTS),
        );
        // TODO: Add label for peer.
        let (network_notifs_tx, network_notifs_rx) = diem_channel::new(
            QueueStyle::FIFO,
            channel_size,
            Some(&counters::PENDING_NETWORK_NOTIFICATIONS),
        );

        let peer = Peer::new(
            Arc::clone(&network_context),
            executor.clone(),
            connection,
            connection_notifs_tx,
            network_reqs_rx,
            network_notifs_tx,
            Duration::from_millis(constants::INBOUND_RPC_TIMEOUT_MS),
            constants::MAX_CONCURRENT_INBOUND_RPCS,
            constants::MAX_CONCURRENT_OUTBOUND_RPCS,
            max_frame_size,
            inbound_rate_limiter,
            outbound_rate_limiter,
        );
        executor.spawn(peer.start());

        (network_reqs_tx, network_notifs_rx)
    }
}
