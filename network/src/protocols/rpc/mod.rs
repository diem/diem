// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Implementation of the RPC protocol as per Libra wire protocol v1.
//!
//! Design:
//! -------
//!
//! RPC receives OutboundRpcRequest messages from upstream actors. The OutboundRpcRequest contains
//! the RPC protocol, raw request bytes, RPC timeout duration, and a channel over which the
//! response bytes can be sent back to the upstream.
//! For inbound RPC requests, RPC sends InboundRpcRequest notifications to upstream actors. The
//! InboundRpcRequest contains the RPC protocol, raw request bytes, and a channel over which the
//! upstream can send a response for the RPC.
//! Internally, the RPC actor consists of a single event loop. The event loop processes 4 kinds of
//! messages:
//! (1) outbound RPC requests received from upstream,
//! (2) notifications for inbound RpcRequest/RpcResponse from the Peer actor,
//! (3) completion notification for tasks processing inbound RPC, and
//! (4) completion notification for tasks processing outbound RPCs.
//! The tasks for inbound and outbound RPCs are spawned onto the same runtime as the one driving
//! the RPC event loop.
//!
//! Timeouts:
//! ---------
//! The tasks for inbound and outbound RPCs are also "wrapped" within timeouts to ensure that they
//! are not running forever. The outbound RPC timeout is specified by the upstream client, where as
//! the inbound RPC timeout is a configuration parameter for the RPC actor.
//!
//! Limits:
//! -------
//! We limit the number of pending inbound RPC tasks to ensure that resource usage is bounded for
//! inbound RPCs. For outbound RPCs, we log a warning when the limit is exceeded, but allow the RPC
//! to proceed.
//!
//! State
//! -------------
//! * For outbound RPCs, the RPC actors maintains a HashMap from the RequestId to a channel over
//! which inbound responses can be delivered to the task driving the request. Entries are removed
//! on completion of the task, which happens either on receipt of the response, or on
//! failure/timeout.
//! * The RPC actor also maintains a RequestIdGenerator for generating request ids for outbound
//! RPCs. The RequestIdGenerator increments the request id by 1 for each subsequent outbound RPC.

use crate::{
    counters,
    peer::{PeerHandle, PeerNotification},
    protocols::wire::messaging::v1::{
        NetworkMessage, Priority, RequestId, RpcRequest, RpcResponse,
    },
    ProtocolId,
};
use bytes::Bytes;
use error::RpcError;
use futures::{
    channel::oneshot,
    future::{self, BoxFuture, FutureExt, TryFutureExt},
    sink::SinkExt,
    stream::{FuturesUnordered, StreamExt},
    task::Context,
};
use libra_logger::prelude::*;
use libra_types::PeerId;
use std::{collections::HashMap, fmt::Debug, time::Duration};

pub mod error;

#[cfg(any(feature = "fuzzing", test))]
#[path = "fuzzing.rs"]
/// fuzzing module for the rpc protocol
pub mod fuzzing;
#[cfg(test)]
mod test;

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

/// Events sent from the [`Rpc`] actor to the
/// [`NetworkProvider`](crate::interface::NetworkProvider) actor.
#[derive(Debug)]
pub enum RpcNotification {
    /// A new inbound rpc request has been received from a remote peer.
    RecvRpc(InboundRpcRequest),
}

type OutboundRpcTasks = FuturesUnordered<BoxFuture<'static, RequestId>>;
type InboundRpcTasks = FuturesUnordered<BoxFuture<'static, ()>>;

// Wraps the task of request id generation. Request ids start at 0 and increment till they hit
// RequestId::MAX. After that, they wrap around to 0.
struct RequestIdGenerator {
    next_id: RequestId,
    peer_id: PeerId,
}

impl RequestIdGenerator {
    pub fn new(peer_id: PeerId) -> Self {
        Self {
            next_id: 0,
            peer_id,
        }
    }

    pub fn next(&mut self) -> RequestId {
        let request_id = self.next_id;
        self.next_id = {
            match self.next_id.overflowing_add(1) {
                (next_id, true) => {
                    info!(
                        "Request ids with peer: {:?} wrapped around to 0",
                        self.peer_id.short_str(),
                    );
                    next_id
                }
                (next_id, _) => next_id,
            }
        };
        request_id
    }
}

/// The rpc actor.
pub struct Rpc {
    /// Channel to send requests to Peer.
    peer_handle: PeerHandle,
    /// Channel to receive requests from other upstream actors.
    requests_rx: channel::Receiver<OutboundRpcRequest>,
    /// Channel to receive notifications from Peer.
    peer_notifs_rx: channel::Receiver<PeerNotification>,
    /// Channels to send notifictions to upstream actors.
    rpc_handler_tx: channel::Sender<RpcNotification>,
    /// The timeout duration for inbound rpc calls.
    inbound_rpc_timeout: Duration,
    /// Channels to send Rpc responses to pending outbound RPC tasks.
    pending_outbound_rpcs: HashMap<RequestId, (ProtocolId, oneshot::Sender<RpcResponse>)>,
    /// RequestId to use for next outbound RPC.
    request_id_gen: RequestIdGenerator,
    /// The maximum number of concurrent outbound rpc requests that we will
    /// service before back-pressure kicks in.
    max_concurrent_outbound_rpcs: u32,
    /// The maximum number of concurrent inbound rpc requests that we will
    /// service before back-pressure kicks in.
    max_concurrent_inbound_rpcs: u32,
}

impl Rpc {
    /// Create a new instance of the [`Rpc`] protocol actor.
    pub fn new(
        peer_handle: PeerHandle,
        requests_rx: channel::Receiver<OutboundRpcRequest>,
        peer_notifs_rx: channel::Receiver<PeerNotification>,
        rpc_handler_tx: channel::Sender<RpcNotification>,
        inbound_rpc_timeout: Duration,
        max_concurrent_outbound_rpcs: u32,
        max_concurrent_inbound_rpcs: u32,
    ) -> Self {
        Self {
            request_id_gen: RequestIdGenerator::new(peer_handle.peer_id()),
            peer_handle,
            requests_rx,
            peer_notifs_rx,
            rpc_handler_tx,
            inbound_rpc_timeout,
            pending_outbound_rpcs: HashMap::new(),
            max_concurrent_outbound_rpcs,
            max_concurrent_inbound_rpcs,
        }
    }

    /// Start the [`Rpc`] actor's event loop.
    pub async fn start(mut self) {
        let mut inbound_rpc_tasks = InboundRpcTasks::new();
        let mut outbound_rpc_tasks = OutboundRpcTasks::new();
        loop {
            ::futures::select! {
                notif = self.peer_notifs_rx.select_next_some() => {
                    self.handle_inbound_message(
                        notif,
                        &mut inbound_rpc_tasks,
                    );
                },
                maybe_req = self.requests_rx.next() => {
                    if let Some(req) = maybe_req {
                        self.handle_outbound_rpc(req, &mut outbound_rpc_tasks).await;
                    } else {
                        break;
                    }
                },
                () = inbound_rpc_tasks.select_next_some() => {
                },
                request_id = outbound_rpc_tasks.select_next_some() => {
                    // Remove request_id from pending_outbound_rpcs if not already removed.
                    let _ = self.pending_outbound_rpcs.remove(&request_id);
                }
            }
        }
        info!(
            "Rpc actor terminated for peer: {}",
            self.peer_handle.peer_id().short_str()
        );
    }

    // Handle inbound message -- the message can be an inbound RPC request, or a response to a
    // pending outbound RPC request.
    fn handle_inbound_message(
        &mut self,
        notif: PeerNotification,
        inbound_rpc_tasks: &mut InboundRpcTasks,
    ) {
        match notif {
            PeerNotification::NewMessage(message) => {
                match message {
                    // This is a response to a pending outbound RPC.
                    NetworkMessage::RpcResponse(response) => {
                        self.handle_inbound_response(response);
                    }
                    // This is a new inbound RPC request.
                    NetworkMessage::RpcRequest(request) => {
                        self.handle_inbound_request(request, inbound_rpc_tasks);
                    }
                    _ => {
                        error!("Received non-RPC message from Peer actor: {:?}", message);
                    }
                }
            }
            notif => debug_assert!(
                false,
                "Received unexpected event from Peer: {:?}, expected NewMessage",
                notif
            ),
        }
    }

    // Handles inbound response by either forwarding response to task waiting for response, or by
    // dropping it if the task has already terminated.
    fn handle_inbound_response(&mut self, response: RpcResponse) {
        let peer_id = self.peer_handle.peer_id();
        let request_id = response.request_id;
        if let Some((protocol, response_tx)) = self.pending_outbound_rpcs.remove(&request_id) {
            trace!(
                "Waiting to notify outbound rpc task about inbound response for request_id {}",
                request_id
            );
            if let Err(e) = response_tx.send(response) {
                warn!(
                    "Failed to handle inbount RPC response from peer: {} for protocol: {:?}. Error: {:?}",
                    peer_id.short_str(),
                    protocol,
                    e
                );
            } else {
                trace!(
                    "Done notifying outbound RPC task about inbound response for request_id {}",
                    request_id
                );
            }
        } else {
            // TODO: add ability to log protocol id as well
            info!(
                "Received response for expired request from {:?}. Discarding.",
                peer_id.short_str()
            )
        }
    }

    // Handle inbound request by spawning task (with timeout).
    fn handle_inbound_request(
        &mut self,
        request: RpcRequest,
        inbound_rpc_tasks: &mut InboundRpcTasks,
    ) {
        let notification_tx = self.rpc_handler_tx.clone();
        let peer_handle = self.peer_handle.clone();
        let peer_id_str = peer_handle.peer_id().short_str();
        if inbound_rpc_tasks.len() as u32 == self.max_concurrent_inbound_rpcs {
            // Increase counter of declined responses and log warning.
            counters::LIBRA_NETWORK_RPC_MESSAGES
                .with_label_values(&["response", "declined"])
                .inc();
            warn!(
                "Pending inbound RPCs are at limit ({}). Not processing new inbound rpc requests",
                self.max_concurrent_inbound_rpcs
            );
            return;
        }
        let timeout = self.inbound_rpc_timeout;
        // Handle request with timeout.
        let f = async move {
            if let Err(err) = tokio::time::timeout(
                timeout,
                handle_inbound_request_inner(notification_tx, request, peer_handle),
            )
            .map_err(Into::<RpcError>::into)
            .map(|r| r.and_then(|x| x))
            .await
            {
                // Log any errors.
                counters::LIBRA_NETWORK_RPC_MESSAGES
                    .with_label_values(&["response", "failed"])
                    .inc();
                warn!(
                    "Error handling inbound rpc request from {}: {:?}",
                    peer_id_str, err
                );
            }
        };
        inbound_rpc_tasks.push(f.boxed());
    }

    /// Handle an outbound rpc request.
    ///
    /// Cancellation is done by the client dropping the receiver side of the [`req.res_tx`]
    /// oneshot channel. If the request is canceled, the rpc future is dropped and the request is
    /// cancelled. Currently, we don't send a cancellation message to the remote peer.
    ///
    /// [`req.res_tx`]: OutboundRpcRequest::res_tx
    async fn handle_outbound_rpc(
        &mut self,
        req: OutboundRpcRequest,
        outbound_rpc_tasks: &mut OutboundRpcTasks,
    ) {
        // If we already have too many pending RPCs, return error immediately.
        if outbound_rpc_tasks.len() as u32 == self.max_concurrent_outbound_rpcs {
            warn!(
                "Pending outbound RPCs ({}) exceeding limit ({}).",
                outbound_rpc_tasks.len(),
                self.max_concurrent_outbound_rpcs,
            );
            let _result = req.res_tx.send(Err(RpcError::TooManyPending(
                self.max_concurrent_outbound_rpcs,
            )));
            return;
        }

        // Unpack request.
        let OutboundRpcRequest {
            protocol,
            data: req_data,
            timeout,
            mut res_tx,
            ..
        } = req;

        let peer_handle = self.peer_handle.clone();
        let peer_id_str = peer_handle.peer_id().short_str();

        // Generate and assign request id to this RPC.
        let request_id = self.request_id_gen.next();

        // Create channel over which response is delivered to future driving outbound RPC.
        let (response_tx, response_rx) = oneshot::channel();
        // Save send end of channel which moving receive end of the channel into the future.
        self.pending_outbound_rpcs
            .insert(request_id, (protocol, response_tx));

        let f = async move {
            // Wrap the outbound rpc protocol with the requested timeout window.
            let mut f_rpc_res = tokio::time::timeout(
                timeout,
                // Future to run the actual outbound rpc protocol.
                handle_outbound_rpc_inner(peer_handle, request_id, protocol, req_data, response_rx),
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
                    if let Err(ref err) = res {
                        counters::LIBRA_NETWORK_RPC_MESSAGES
                            .with_label_values(&["request", "failed"])
                            .inc();
                        warn!(
                            "Error making outbound rpc request with request_id {} to {}: {:?}",
                            request_id, peer_id_str, err
                        );
                    }
                    // Propagate the results to the rpc client layer.
                    if res_tx.send(res).is_err() {
                        counters::LIBRA_NETWORK_RPC_MESSAGES
                            .with_label_values(&["request", "cancelled"])
                            .inc();
                        info!("Rpc client canceled outbound rpc call to {}", peer_id_str);
                    }
                },
                // The rpc client canceled the request
                cancel = f_rpc_cancel => {
                    counters::LIBRA_NETWORK_RPC_MESSAGES
                        .with_label_values(&["request", "cancelled"])
                        .inc();
                    info!("Rpc client canceled outbound rpc call to {}", peer_id_str);
                },
            }
            // Return the request_id for state management in the main event-loop.
            request_id
        };
        outbound_rpc_tasks.push(f.boxed());
    }
}

async fn handle_outbound_rpc_inner(
    mut peer_handle: PeerHandle,
    request_id: RequestId,
    protocol: ProtocolId,
    req_data: Bytes,
    response_rx: oneshot::Receiver<RpcResponse>,
) -> Result<Bytes, RpcError> {
    let req_len = req_data.len();
    let peer_id = peer_handle.peer_id();

    // Create NetworkMessage to be sent over the wire.
    let request = NetworkMessage::RpcRequest(RpcRequest {
        request_id,
        // TODO: Use default priority for now. To be exposed via network API.
        priority: Priority::default(),
        protocol_id: protocol,
        raw_request: Vec::from(req_data.as_ref()),
    });

    // Start timer to collect RPC latency.
    let _timer = counters::LIBRA_NETWORK_RPC_LATENCY.start_timer();

    // Send outbound request to peer_handle.
    trace!(
        "Sending outbound rpc request with request_id {} to peer: {:?}",
        request_id,
        peer_id.short_str()
    );
    peer_handle.send_message(request, protocol).await?;

    // Collect counters for requests sent.
    counters::LIBRA_NETWORK_RPC_MESSAGES
        .with_label_values(&["request", "sent"])
        .inc();
    counters::LIBRA_NETWORK_RPC_BYTES
        .with_label_values(&["request", "sent"])
        .observe(req_len as f64);

    // Wait for listener's response.
    trace!(
        "Waiting to receive response for request_id {} from peer: {:?}",
        request_id,
        peer_id.short_str()
    );
    let response = response_rx.await?;
    trace!(
        "Received response for request_id {} from peer: {:?}",
        request_id,
        peer_id.short_str()
    );

    // Collect counters for received response.
    let res_data = response.raw_response;
    counters::LIBRA_NETWORK_RPC_MESSAGES
        .with_label_values(&["response", "received"])
        .inc();
    counters::LIBRA_NETWORK_RPC_BYTES
        .with_label_values(&["response", "received"])
        .observe(res_data.len() as f64);
    Ok(Bytes::from(res_data))
}

async fn handle_inbound_request_inner(
    mut notification_tx: channel::Sender<RpcNotification>,
    request: RpcRequest,
    mut peer_handle: PeerHandle,
) -> Result<(), RpcError> {
    let req_data = request.raw_request;
    let request_id = request.request_id;
    let peer_id = peer_handle.peer_id();

    trace!(
        "Received inbound request with request_id {} from peer: {:?}",
        request_id,
        peer_id.short_str()
    );
    // Collect counters for received request.
    counters::LIBRA_NETWORK_RPC_MESSAGES
        .with_label_values(&["request", "received"])
        .inc();
    counters::LIBRA_NETWORK_RPC_BYTES
        .with_label_values(&["request", "received"])
        .observe(req_data.len() as f64);

    // Forward request to upper layer.
    let (res_tx, res_rx) = oneshot::channel();
    let notification = RpcNotification::RecvRpc(InboundRpcRequest {
        protocol: request.protocol_id,
        data: Bytes::from(req_data),
        res_tx,
    });
    notification_tx.send(notification).await?;

    // Wait for response from upper layer.
    trace!(
        "Waiting for upstream response for inbound request with request_id {} from peer: {:?}",
        request_id,
        peer_id.short_str()
    );
    let res_data = res_rx.await??;
    let res_len = res_data.len();

    // Send response to remote peer.
    trace!(
        "Sending response for request_id {} to peer: {:?}",
        request_id,
        peer_id.short_str()
    );
    let response = RpcResponse {
        raw_response: Vec::from(res_data.as_ref()),
        request_id,
        priority: request.priority,
    };
    peer_handle
        .send_message(NetworkMessage::RpcResponse(response), request.protocol_id)
        .await?;

    // Collect counters for sent response.
    counters::LIBRA_NETWORK_RPC_MESSAGES
        .with_label_values(&["response", "sent"])
        .inc();
    counters::LIBRA_NETWORK_RPC_BYTES
        .with_label_values(&["response", "sent"])
        .observe(res_len as f64);
    Ok(())
}
