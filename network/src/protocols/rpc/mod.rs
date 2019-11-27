// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Protocol for making and handling Remote Procedure Calls
//!
//! # SLURP: Simple Libra Unary Rpc Protocol
//!
//! SLURP takes advantage of [muxers] and [substream negotiation] to build a
//! simple rpc protocol. Concretely,
//!
//! 1. Every rpc call runs in its own substream. Instead of managing a completion
//!    queue of message ids, we instead delegate this handling to the muxer so
//!    that the underlying substream controls the lifetime of the rpc call.
//!    Additionally, on certain transports (e.g., QUIC) we avoid head-of-line
//!    blocking as the substreams are independent.
//! 2. An rpc method call negotiates which method to call using [`protocol-select`].
//!    This allows simple versioning of rpc methods and negotiation of which
//!    methods are supported. In the future, we can potentially support multiple
//!    backwards-incompatible versions of any rpc method.
//! 3. The actual structure of the request/response wire messages is left for
//!    higher layers to specify. The rpc protocol is only concerned with shipping
//!    around opaque blobs. Current libra rpc clients (consensus, mempool) mostly
//!    send protobuf enums around over a single rpc protocol,
//!    e.g., `/libra/rpc/0.1.0/consensus/0.1.0`.
//!
//! ## Wire Protocol (dialer):
//!
//! To make an rpc request to a remote peer, the dialer
//!
//! 1. Requests a new outbound substream from the muxer.
//! 2. Negotiates the substream using [`protocol-select`] to the rpc method they
//!    wish to call, e.g., `/libra/rpc/0.1.0/mempool/0.10`.
//! 3. Sends the serialized request arguments on the newly negotiated substream.
//! 4. Half-closes their output side.
//! 5. Awaits the serialized response message from remote.
//! 6. Awaits the listener's half-close to complete the substream close.
//!
//! ## Wire Protocol (listener):
//!
//! To handle new rpc requests from remote peers, the listener
//!
//! 1. Polls for new inbound substreams on the muxer.
//! 2. Negotiates inbound substreams using [`protocol-select`]. The negotiation
//!    must only succeed if the requested rpc method is actually supported.
//! 3. Awaits the serialized request arguments on the newly negotiated substream.
//! 4. Awaits the dialer's half-close.
//! 5. Handles the request by sending it up through the
//!    [`NetworkProvider`](crate::interface::NetworkProvider)
//!    actor to a higher layer rpc client like consensus or mempool, who then
//!    sends the serialed rpc response back down to the rpc layer.
//! 6. Sends the serialized response message to the dialer.
//! 7. Half-closes their output side to complete the substream close.
//!
//! [muxers]: ../../../netcore/multiplexing/index.html
//! [substream negotiation]: ../../../netcore/negotiate/index.html
//! [`protocol-select`]: ../../../netcore/negotiate/index.html

use crate::{
    counters,
    peer_manager::{PeerManagerNotification, PeerManagerRequestSender},
    sink::NetworkSinkExt,
    ProtocolId,
};
use bounded_executor::BoundedExecutor;
use bytes05::Bytes;
use channel;
use error::RpcError;
use futures::{
    channel::oneshot,
    future::{self, FutureExt, TryFutureExt},
    io::{AsyncRead, AsyncWrite},
    sink::SinkExt,
    stream::StreamExt,
    task::Context,
};
use libra_logger::prelude::*;
use libra_types::PeerId;
use netcore::compat::IoCompat;
use std::{fmt::Debug, io, time::Duration};
use tokio::runtime::Handle;
use tokio_util::codec::{Framed, LengthDelimitedCodec};

pub mod error;
pub mod utils;

#[cfg(test)]
mod test;

#[cfg(any(feature = "fuzzing", test))]
#[path = "fuzzing.rs"]
/// fuzzing module for the rpc protocol
pub mod fuzzing;

/// A wrapper struct for an inbound rpc request and its associated context.
#[derive(Debug)]
pub struct InboundRpcRequest {
    /// Rpc method identifier, e.g., `/libra/rpc/0.1.0/consensus/0.1.0`. This is used
    /// to dispatch the request to the corresponding client handler.
    pub protocol: ProtocolId,
    /// The serialized request data received from the sender.
    pub data: Bytes,
    /// Channel over which the rpc response is sent from the upper client layer
    /// to the rpc layer.
    ///
    /// The rpc actor holds onto the receiving end of this channel, awaiting the
    /// response from the upper layer. If there is an error in, e.g.,
    /// deserializing the request, the upper layer should send an [`RpcError`]
    /// down the channel to signify that there was an error while handling this
    /// rpc request. Currently, we just log these errors and drop the substream;
    /// in the future, we will send an error response to the peer and/or log any
    /// malicious behaviour.
    ///
    /// The upper client layer should be prepared for `res_tx` to be potentially
    /// disconnected when trying to send their response, as the rpc call might
    /// have timed out while handling the request.
    pub res_tx: oneshot::Sender<Result<Bytes, RpcError>>,
}

/// A wrapper struct for an outbound rpc request and its associated context.
#[derive(Debug)]
pub struct OutboundRpcRequest {
    /// Rpc method identifier, e.g., `/libra/rpc/0.1.0/consensus/0.1.0`. This is the
    /// protocol we will negotiate our outbound substream to.
    pub protocol: ProtocolId,
    /// The serialized request data to be sent to the receiver.
    pub data: Bytes,
    /// Channel over which the rpc response is sent from the rpc layer to the
    /// upper client layer.
    ///
    /// If there is an error while performing the rpc protocol, e.g., the remote
    /// peer drops the connection, we will send an [`RpcError`] over the channel.
    pub res_tx: oneshot::Sender<Result<Bytes, RpcError>>,
    /// The timeout duration for the entire rpc call. If the timeout elapses, the
    /// rpc layer will send an [`RpcError::TimedOut`] error over the
    /// `res_tx` channel to the upper client layer.
    pub timeout: Duration,
}

/// Events sent from the [`NetworkProvider`](crate::interface::NetworkProvider)
/// actor to the [`Rpc`] actor.
#[derive(Debug)]
pub enum RpcRequest {
    /// Send an outbound rpc request to a remote peer.
    SendRpc(PeerId, OutboundRpcRequest),
}

/// Events sent from the [`Rpc`] actor to the
/// [`NetworkProvider`](crate::interface::NetworkProvider) actor.
#[derive(Debug)]
pub enum RpcNotification {
    /// A new inbound rpc request has been received from a remote peer.
    RecvRpc(PeerId, InboundRpcRequest),
}

/// The rpc actor.
pub struct Rpc<TSubstream> {
    /// Executor to spawn inbound and outbound handler tasks.
    executor: Handle,
    /// Channel to receive requests from other upstream actors.
    requests_rx: channel::Receiver<RpcRequest>,
    /// Channel to receive notifications from [`PeerManager`](crate::peer_manager::PeerManager).
    peer_mgr_notifs_rx: channel::Receiver<PeerManagerNotification<TSubstream>>,
    /// Channel to send requests to [`PeerManager`](crate::peer_manager::PeerManager).
    peer_mgr_reqs_tx: PeerManagerRequestSender<TSubstream>,
    /// Channels to send notifictions to upstream actors.
    rpc_handler_tx: channel::Sender<RpcNotification>,
    /// The timeout duration for inbound rpc calls.
    inbound_rpc_timeout: Duration,
    /// The maximum number of concurrent outbound rpc requests that we will
    /// service before back-pressure kicks in.
    max_concurrent_outbound_rpcs: u32,
    /// The maximum number of concurrent inbound rpc requests that we will
    /// service before back-pressure kicks in.
    // TODO(philiphayes): partition inbound queue by peer to prevent one peer
    // from starving other peers' rpcs?
    max_concurrent_inbound_rpcs: u32,
}

impl<TSubstream> Rpc<TSubstream>
where
    TSubstream: AsyncRead + AsyncWrite + Send + Unpin + Debug + 'static,
{
    /// Create a new instance of the [`Rpc`] protocol actor.
    pub fn new(
        executor: Handle,
        requests_rx: channel::Receiver<RpcRequest>,
        peer_mgr_notifs_rx: channel::Receiver<PeerManagerNotification<TSubstream>>,
        peer_mgr_reqs_tx: PeerManagerRequestSender<TSubstream>,
        rpc_handler_tx: channel::Sender<RpcNotification>,
        inbound_rpc_timeout: Duration,
        max_concurrent_outbound_rpcs: u32,
        max_concurrent_inbound_rpcs: u32,
    ) -> Self {
        Self {
            executor,
            requests_rx,
            peer_mgr_notifs_rx,
            peer_mgr_reqs_tx,
            rpc_handler_tx,
            inbound_rpc_timeout,
            max_concurrent_outbound_rpcs,
            max_concurrent_inbound_rpcs,
        }
    }

    /// Start the [`Rpc`] actor's event loop.
    pub async fn start(self) {
        // unpack self to satisfy borrow checker
        let executor = self.executor;
        let requests_rx = self.requests_rx;
        let peer_mgr_notifs_rx = self.peer_mgr_notifs_rx;
        let peer_mgr_reqs_tx = self.peer_mgr_reqs_tx;
        let rpc_handler_tx = self.rpc_handler_tx;
        let inbound_rpc_timeout = self.inbound_rpc_timeout;
        let max_concurrent_outbound_rpcs = self.max_concurrent_outbound_rpcs;
        let max_concurrent_inbound_rpcs = self.max_concurrent_inbound_rpcs;

        // inbound and outbound requests use separate bounded executors to ensure
        // backpressure propagates independently and doesn't starve the other
        // handler.

        let outbound_handler = handle_outbounds(
            BoundedExecutor::new(max_concurrent_outbound_rpcs as usize, executor.clone()),
            requests_rx,
            peer_mgr_reqs_tx,
        );

        let inbound_handler = handle_inbounds(
            BoundedExecutor::new(max_concurrent_inbound_rpcs as usize, executor),
            peer_mgr_notifs_rx,
            rpc_handler_tx,
            inbound_rpc_timeout,
        );

        // drive inbound and outbound handlers to completion
        future::join(outbound_handler, inbound_handler).await;

        crit!("Rpc actor terminated");
    }
}

/// Handle all outbound rpcs.
async fn handle_outbounds<TSubstream>(
    executor: BoundedExecutor,
    mut requests_rx: channel::Receiver<RpcRequest>,
    peer_mgr_tx: PeerManagerRequestSender<TSubstream>,
) where
    TSubstream: AsyncRead + AsyncWrite + Send + Unpin + 'static,
{
    while let Some(req) = requests_rx.next().await {
        executor
            .spawn(handle_outbound_rpc(peer_mgr_tx.clone(), req))
            .await;
    }
}

/// Handle all inbound rpcs.
async fn handle_inbounds<TSubstream>(
    executor: BoundedExecutor,
    mut peer_mgr_notifs_rx: channel::Receiver<PeerManagerNotification<TSubstream>>,
    rpc_handler_tx: channel::Sender<RpcNotification>,
    inbound_rpc_timeout: Duration,
) where
    TSubstream: AsyncRead + AsyncWrite + Debug + Send + Unpin + 'static,
{
    while let Some(notif) = peer_mgr_notifs_rx.next().await {
        executor
            .spawn(handle_inbound_substream(
                rpc_handler_tx.clone(),
                notif,
                inbound_rpc_timeout,
            ))
            .await;
    }
}

/// Handle an outbound rpc request event. Open a new substream then run the
/// outbound rpc protocol over the substream.
///
/// The request results (including errors) are propagated up to the rpc client
/// through the [`req.res_tx`] oneshot channel. Cancellation is done by the client
/// dropping the receiver side of the [`req.res_tx`] oneshot channel. If the
/// request is canceled, the substream will be dropped and a RST frame will be
/// sent over the muxer closing the substream.
///
/// [`req.res_tx`]: OutboundRpcRequest::res_tx
async fn handle_outbound_rpc<TSubstream>(
    peer_mgr_tx: PeerManagerRequestSender<TSubstream>,
    req: RpcRequest,
) where
    TSubstream: AsyncRead + AsyncWrite + Send + Unpin,
{
    match req {
        RpcRequest::SendRpc(peer_id, req) => {
            let protocol = req.protocol;
            let req_data = req.data;
            let mut res_tx = req.res_tx;
            let timeout = req.timeout;

            // Future to run the actual outbound rpc protocol and get the results.
            let mut f_rpc_res = tokio::time::timeout(
                timeout,
                handle_outbound_rpc_inner(peer_mgr_tx, peer_id, protocol, req_data),
            )
            .map_err(Into::<RpcError>::into)
            .map(|r| r.and_then(|x| x))
            .boxed()
            .fuse();

            // If the rpc client drops their oneshot receiver, this future should
            // cancel the request.
            let mut f_rpc_cancel =
                future::poll_fn(|cx: &mut Context| res_tx.poll_canceled(cx)).fuse();

            futures::select! {
                res = f_rpc_res => {
                    // Log any errors.
                    if let Err(err) = &res {
                        counters::LIBRA_NETWORK_RPC_MESSAGES
                            .with_label_values(&["request", "failed"])
                            .inc();
                        warn!(
                            "Error making outbound rpc request to {}: {:?}",
                            peer_id.short_str(), err
                        );
                    }

                    // Propagate the results to the rpc client layer.
                    if res_tx.send(res).is_err() {
                        counters::LIBRA_NETWORK_RPC_MESSAGES
                            .with_label_values(&["request", "cancelled"])
                            .inc();
                        debug!("Rpc client canceled outbound rpc call to {}", peer_id.short_str());
                    }
                },
                // The rpc client canceled the request
                cancel = f_rpc_cancel => {
                    counters::LIBRA_NETWORK_RPC_MESSAGES
                        .with_label_values(&["request", "cancelled"])
                        .inc();
                    debug!("Rpc client canceled outbound rpc call to {}", peer_id.short_str());
                },
            }
        }
    }
}

async fn handle_outbound_rpc_inner<TSubstream>(
    mut peer_mgr_tx: PeerManagerRequestSender<TSubstream>,
    peer_id: PeerId,
    protocol: ProtocolId,
    req_data: Bytes,
) -> Result<Bytes, RpcError>
where
    TSubstream: AsyncRead + AsyncWrite + Send + Unpin,
{
    let _timer = counters::LIBRA_NETWORK_RPC_LATENCY.start_timer();
    // Request a new substream with the peer.
    let substream = peer_mgr_tx.open_substream(peer_id, protocol).await?;
    // Rpc messages are length-prefixed.
    let mut substream = Framed::new(IoCompat::new(substream), LengthDelimitedCodec::new());
    // Send the rpc request data.
    let req_len = req_data.len();
    substream.buffered_send(req_data).await?;
    // We won't send anything else on this substream, so we can half-close our
    // output side.
    substream.close().await?;
    counters::LIBRA_NETWORK_RPC_MESSAGES
        .with_label_values(&["request", "sent"])
        .inc();
    counters::LIBRA_NETWORK_RPC_BYTES
        .with_label_values(&["request", "sent"])
        .observe(req_len as f64);

    // Wait for listener's response.
    let res_data = match substream.next().await {
        Some(res_data) => res_data?.freeze(),
        None => return Err(io::Error::from(io::ErrorKind::UnexpectedEof).into()),
    };

    // Wait for listener to half-close their side.
    match substream.next().await {
        // Remote should never send more than one response; we'll consider this
        // a protocol violation and ignore their response.
        Some(_) => Err(RpcError::UnexpectedRpcResponse),
        None => Ok(res_data),
    }
}

/// Handle an new inbound substream. Run the inbound rpc protocol over the
/// substream.
async fn handle_inbound_substream<TSubstream>(
    notification_tx: channel::Sender<RpcNotification>,
    notif: PeerManagerNotification<TSubstream>,
    timeout: Duration,
) where
    TSubstream: AsyncRead + AsyncWrite + Debug + Send + Unpin,
{
    match notif {
        PeerManagerNotification::NewInboundSubstream(peer_id, substream) => {
            // Run the actual inbound rpc protocol.
            let res = tokio::time::timeout(
                timeout,
                handle_inbound_substream_inner(
                    notification_tx,
                    peer_id,
                    substream.protocol,
                    substream.substream,
                ),
            )
            .map_err(Into::<RpcError>::into)
            .map(|r| r.and_then(|x| x))
            .await;

            // Log any errors.
            if let Err(err) = res {
                counters::LIBRA_NETWORK_RPC_MESSAGES
                    .with_label_values(&["response", "failed"])
                    .inc();
                warn!(
                    "Error handling inbound rpc request from {}: {:?}",
                    peer_id.short_str(),
                    err
                );
            }
        }
        notif => debug_assert!(
            false,
            "Received unexpected event from PeerManager: {:?}, expected NewInboundSubstream",
            notif
        ),
    }
}

async fn handle_inbound_substream_inner<TSubstream>(
    mut notification_tx: channel::Sender<RpcNotification>,
    peer_id: PeerId,
    protocol: ProtocolId,
    substream: TSubstream,
) -> Result<(), RpcError>
where
    TSubstream: AsyncRead + AsyncWrite + Send + Unpin,
{
    // Rpc messages are length-prefixed.
    let mut substream = Framed::new(IoCompat::new(substream), LengthDelimitedCodec::new());
    // Read the rpc request data.
    let req_data = match substream.next().await {
        Some(req_data) => req_data?.freeze(),
        None => return Err(io::Error::from(io::ErrorKind::UnexpectedEof).into()),
    };
    counters::LIBRA_NETWORK_RPC_MESSAGES
        .with_label_values(&["request", "received"])
        .inc();

    // Wait for dialer to half-close their side.
    if substream.next().await.is_some() {
        // Remote should never send more than one request; we'll consider this
        // a protocol violation and ignore their request.
        return Err(RpcError::UnexpectedRpcRequest);
    };

    // Build the event and context we push up to upper layers for handling.
    let (res_tx, res_rx) = oneshot::channel();
    let notification = RpcNotification::RecvRpc(
        peer_id,
        InboundRpcRequest {
            protocol,
            data: req_data,
            res_tx,
        },
    );
    // TODO(philiphayes): impl correct shutdown process so this never panics
    // Forward request to upper layer.
    notification_tx.send(notification).await.unwrap();

    // Wait for response from upper layer.
    let res_data = res_rx.await??;
    let res_len = res_data.len();

    // Send the response to remote
    substream.buffered_send(res_data).await?;

    // We won't send anything else on this substream, so we can half-close
    // our output. The initiator will have also half-closed their side before
    // this, so this should gracefully shutdown the socket.
    substream.close().await?;
    counters::LIBRA_NETWORK_RPC_MESSAGES
        .with_label_values(&["response", "sent"])
        .inc();
    counters::LIBRA_NETWORK_RPC_BYTES
        .with_label_values(&["response", "sent"])
        .observe(res_len as f64);

    Ok(())
}
