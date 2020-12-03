// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Implementation of the RPC protocol as per Diem wire protocol v1.
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
    counters::{
        self, inc_by_with_context, CANCELED_LABEL, DECLINED_LABEL, FAILED_LABEL, RECEIVED_LABEL,
        REQUEST_LABEL, RESPONSE_LABEL, SENT_LABEL,
    },
    logging::NetworkSchema,
    peer::{PeerHandle, PeerNotification},
    protocols::wire::messaging::v1::{
        NetworkMessage, Priority, RequestId, RpcRequest, RpcResponse,
    },
    ProtocolId,
};
use bytes::Bytes;
use diem_config::network_id::NetworkContext;
use diem_logger::prelude::*;
use diem_types::PeerId;
use error::RpcError;
use futures::{
    channel::oneshot,
    future::{self, BoxFuture, FutureExt, TryFutureExt},
    sink::SinkExt,
    stream::{FuturesUnordered, StreamExt},
    task::Context,
};
use serde::Serialize;
use std::{collections::HashMap, fmt::Debug, sync::Arc, time::Duration};

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
    /// The [`ProtocolId`] for which of our upstream application modules should
    /// handle (i.e., deserialize and then respond to) this inbound rpc request.
    ///
    /// For example, if `protocol_id == ProtocolId::ConsensusRpc`, then this
    /// inbound rpc request will be dispatched to consensus for handling.
    pub protocol_id: ProtocolId,
    /// The serialized request data received from the sender. At this layer in
    /// the stack, the request data is just an opaque blob and will only be fully
    /// deserialized later in the handling application module.
    pub data: Bytes,
    /// Channel over which the rpc response is sent from the upper application
    /// layer to the network rpc layer.
    ///
    /// The rpc actor holds onto the receiving end of this channel, awaiting the
    /// response from the upper layer. If there is an error in, e.g.,
    /// deserializing the request, the upper layer should send an [`RpcError`]
    /// down the channel to signify that there was an error while handling this
    /// rpc request. Currently, we just log these errors and drop the request.
    ///
    /// The upper client layer should be prepared for `res_tx` to be disconnected
    /// when trying to send their response, as the rpc call might have timed out
    /// while handling the request.
    pub res_tx: oneshot::Sender<Result<Bytes, RpcError>>,
}

/// A wrapper struct for an outbound rpc request and its associated context.
#[derive(Debug, Serialize)]
pub struct OutboundRpcRequest {
    /// The remote peer's application module that should handle our outbound rpc
    /// request.
    ///
    /// For example, if `protocol_id == ProtocolId::ConsensusRpc`, then this
    /// outbound rpc request should be handled by the remote peer's consensus
    /// application module.
    pub protocol_id: ProtocolId,
    /// The serialized request data to be sent to the receiver. At this layer in
    /// the stack, the request data is just an opaque blob.
    #[serde(skip)]
    pub data: Bytes,
    /// Channel over which the rpc response is sent from the rpc layer to the
    /// upper client layer.
    ///
    /// If there is an error while performing the rpc protocol, e.g., the remote
    /// peer drops the connection, we will send an [`RpcError`] over the channel.
    #[serde(skip)]
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
                        remote_peer = self.peer_id,
                        "Request ids with peer: {} wrapped around to 0",
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
    /// The network instance this Rpc actor is running under.
    network_context: Arc<NetworkContext>,
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
        network_context: Arc<NetworkContext>,
        peer_handle: PeerHandle,
        requests_rx: channel::Receiver<OutboundRpcRequest>,
        peer_notifs_rx: channel::Receiver<PeerNotification>,
        rpc_handler_tx: channel::Sender<RpcNotification>,
        inbound_rpc_timeout: Duration,
        max_concurrent_outbound_rpcs: u32,
        max_concurrent_inbound_rpcs: u32,
    ) -> Self {
        Self {
            network_context,
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
        let peer_id = self.peer_handle.peer_id();
        let mut inbound_rpc_tasks = InboundRpcTasks::new();
        let mut outbound_rpc_tasks = OutboundRpcTasks::new();

        trace!(
            NetworkSchema::new(&self.network_context).remote_peer(&peer_id),
            "{} Rpc actor for '{}' started",
            self.network_context,
            peer_id.short_str()
        );
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
        trace!(
            NetworkSchema::new(&self.network_context).remote_peer(&peer_id),
            "{} Rpc actor for '{}' terminated",
            self.network_context,
            peer_id.short_str()
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
                        inc_by_with_context(
                            &counters::INVALID_NETWORK_MESSAGES,
                            &self.network_context,
                            "rpc",
                            1,
                        );
                        error!(
                            NetworkSchema::new(&self.network_context)
                                .remote_peer(&self.peer_handle.peer_id()),
                            "{} Received non-RPC message from Peer actor: {:?}",
                            self.network_context,
                            message
                        );
                    }
                }
            }
            notif => {
                inc_by_with_context(
                    &counters::INVALID_NETWORK_MESSAGES,
                    &self.network_context,
                    "rpc",
                    1,
                );
                error!(
                    NetworkSchema::new(&self.network_context)
                        .remote_peer(&self.peer_handle.peer_id()),
                    "{} Received unexpected event from Peer: {:?}, expected NewMessage",
                    self.network_context,
                    notif
                )
            }
        }
    }

    // Handles inbound response by either forwarding response to task waiting for response, or by
    // dropping it if the task has already terminated.
    fn handle_inbound_response(&mut self, response: RpcResponse) {
        let peer_id = self.peer_handle.peer_id();
        let request_id = response.request_id;
        if let Some((protocol_id, response_tx)) = self.pending_outbound_rpcs.remove(&request_id) {
            trace!(
                NetworkSchema::new(&self.network_context),
                request_id = request_id,
                "{} Waiting to notify outbound rpc task about inbound response for request_id {}",
                self.network_context,
                request_id
            );
            if let Err(e) = response_tx.send(response) {
                warn!(
                    NetworkSchema::new(&self.network_context)
                        .remote_peer(&peer_id),
                    error = ?e,
                    protocol_id = protocol_id,
                    "{} Failed to handle inbound RPC response from peer: {} for protocol: {}. Error: {:?}",
                    self.network_context,
                    peer_id.short_str(),
                    protocol_id,
                    e
                );
            } else {
                trace!(
                    NetworkSchema::new(&self.network_context),
                    request_id = request_id,
                    "{} Done notifying outbound RPC task about inbound response for request_id {}",
                    self.network_context,
                    request_id
                );
            }
        } else {
            // TODO: add ability to log protocol id as well
            info!(
                NetworkSchema::new(&self.network_context).remote_peer(&peer_id),
                "{} Received response for expired request from {}. Discarding.",
                self.network_context,
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
        let network_context = Arc::clone(&self.network_context);

        // Drop new inbound requests if our completion queue is at capacity.
        if inbound_rpc_tasks.len() as u32 == self.max_concurrent_inbound_rpcs {
            // Increase counter of declined responses and log warning.
            counters::rpc_messages(&network_context, RESPONSE_LABEL, DECLINED_LABEL).inc();
            warn!(
                NetworkSchema::new(&self.network_context),
                "{} Pending inbound RPCs are at limit ({}). Not processing new inbound rpc requests",
                self.network_context,
                self.max_concurrent_inbound_rpcs
            );
            return;
        }

        let notification_tx = self.rpc_handler_tx.clone();
        let peer_handle = self.peer_handle.clone();
        let peer_id = peer_handle.peer_id();
        let timeout = self.inbound_rpc_timeout;

        // Handle request with timeout.
        let f = async move {
            if let Err(err) = tokio::time::timeout(
                timeout,
                handle_inbound_request_inner(
                    &network_context,
                    notification_tx,
                    request,
                    peer_handle,
                ),
            )
            .map_err(Into::<RpcError>::into)
            .map(|r| r.and_then(|x| x))
            .await
            {
                // Log any errors.
                counters::rpc_messages(&network_context, RESPONSE_LABEL, FAILED_LABEL).inc();
                warn!(
                    NetworkSchema::new(&network_context)
                        .remote_peer(&peer_id),
                    error = ?err,
                    "{} Error handling inbound rpc request from {}: {:?}",
                    network_context,
                    peer_id.short_str(),
                    err
                );
            }
        };
        inbound_rpc_tasks.push(f.boxed());
    }

    /// Handle an outbound rpc request.
    ///
    /// Cancellation is done by the client dropping the receiver side of the [`req.res_tx`]
    /// oneshot channel. If the request is canceled, the rpc future is dropped and the request is
    /// canceled. Currently, we don't send a cancellation message to the remote peer.
    ///
    /// [`req.res_tx`]: OutboundRpcRequest::res_tx
    async fn handle_outbound_rpc(
        &mut self,
        req: OutboundRpcRequest,
        outbound_rpc_tasks: &mut OutboundRpcTasks,
    ) {
        let network_context = Arc::clone(&self.network_context);

        // Unpack request.
        let OutboundRpcRequest {
            protocol_id,
            data: req_data,
            timeout,
            mut res_tx,
            ..
        } = req;

        // If we already have too many pending RPCs, return error immediately.
        if outbound_rpc_tasks.len() as u32 == self.max_concurrent_outbound_rpcs {
            counters::rpc_messages(&network_context, REQUEST_LABEL, DECLINED_LABEL).inc();
            warn!(
                NetworkSchema::new(&self.network_context),
                "{} Pending outbound RPCs ({}) exceeding limit ({}).",
                self.network_context,
                outbound_rpc_tasks.len(),
                self.max_concurrent_outbound_rpcs,
            );
            let _result = res_tx.send(Err(RpcError::TooManyPending(
                self.max_concurrent_outbound_rpcs,
            )));
            return;
        }

        let peer_handle = self.peer_handle.clone();
        let peer_id = peer_handle.peer_id();

        // Generate and assign request id to this RPC.
        let request_id = self.request_id_gen.next();

        // Create channel over which response is delivered to future driving outbound RPC.
        let (response_tx, response_rx) = oneshot::channel();
        // Save send end of channel which moving receive end of the channel into the future.
        self.pending_outbound_rpcs
            .insert(request_id, (protocol_id, response_tx));

        let f = async move {
            // Wrap the outbound rpc protocol with the requested timeout window.
            let mut f_rpc_res = tokio::time::timeout(
                timeout,
                // Future to run the actual outbound rpc protocol.
                handle_outbound_rpc_inner(
                    &network_context,
                    peer_handle,
                    request_id,
                    protocol_id,
                    req_data,
                    response_rx,
                ),
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
                        counters::rpc_messages(&network_context, REQUEST_LABEL, FAILED_LABEL)
                            .inc();
                        warn!(
                            NetworkSchema::new(&network_context)
                                .remote_peer(&peer_id),
                            error = ?err,
                            request_id = request_id,
                            "{} Error making outbound rpc request with request_id {} to {}: {:?}",
                            network_context,
                            request_id,
                            peer_id.short_str(),
                            err
                        );
                    }
                    // Propagate the results to the rpc client layer.
                    if res_tx.send(res).is_err() {
                        counters::rpc_messages(&network_context, REQUEST_LABEL, CANCELED_LABEL)
                            .inc();
                        info!(
                            NetworkSchema::new(&network_context)
                                .remote_peer(&peer_id),
                            "{} Rpc client canceled outbound rpc call to {}",
                            network_context,
                            peer_id.short_str()
                        );
                    }
                },
                // The rpc client canceled the request
                _cancel = f_rpc_cancel => {
                    counters::rpc_messages(&network_context, REQUEST_LABEL, CANCELED_LABEL)
                        .inc();
                    info!(
                        NetworkSchema::new(&network_context)
                            .remote_peer(&peer_id),
                        "{} Rpc client canceled outbound rpc call to {}",
                        network_context,
                        peer_id.short_str()
                    );
                },
            }
            // Return the request_id for state management in the main event-loop.
            request_id
        };
        outbound_rpc_tasks.push(f.boxed());
    }
}

async fn handle_outbound_rpc_inner(
    network_context: &NetworkContext,
    mut peer_handle: PeerHandle,
    request_id: RequestId,
    protocol_id: ProtocolId,
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
        protocol_id,
        raw_request: Vec::from(req_data.as_ref()),
    });

    // Send outbound request to peer_handle.
    trace!(
        NetworkSchema::new(&network_context).remote_peer(&peer_id),
        request_id = request_id,
        "{} Sending outbound rpc request with request_id {} and protocol_id {} to peer {}",
        network_context,
        request_id,
        protocol_id,
        peer_id.short_str(),
    );
    // Start timer to collect RPC latency.
    let timer = counters::outbound_rpc_request_latency(network_context, protocol_id).start_timer();
    peer_handle.send_message(request, protocol_id).await?;

    // Collect counters for requests sent.
    counters::rpc_messages(network_context, REQUEST_LABEL, SENT_LABEL).inc();
    counters::rpc_bytes(network_context, REQUEST_LABEL, SENT_LABEL).inc_by(req_len as u64);

    // Wait for listener's response.
    trace!(
        NetworkSchema::new(&network_context).remote_peer(&peer_id),
        request_id = request_id,
        "{} Waiting to receive response for request_id {} and protocol_id {} from peer {}",
        network_context,
        request_id,
        protocol_id,
        peer_id.short_str(),
    );
    let response = response_rx.await?;
    let latency = timer.stop_and_record();
    trace!(
        NetworkSchema::new(&network_context).remote_peer(&peer_id),
        request_id = request_id,
        protocol_id = protocol_id,
        "{} Received response for request_id {} and protocol_id {} from peer {} \
        with {:.6} seconds of latency",
        network_context,
        request_id,
        protocol_id,
        peer_id.short_str(),
        latency,
    );

    // Collect counters for received response.
    let res_data = response.raw_response;
    counters::rpc_messages(network_context, RESPONSE_LABEL, RECEIVED_LABEL).inc();
    counters::rpc_bytes(network_context, RESPONSE_LABEL, RECEIVED_LABEL)
        .inc_by(res_data.len() as u64);
    Ok(Bytes::from(res_data))
}

async fn handle_inbound_request_inner(
    network_context: &NetworkContext,
    mut notification_tx: channel::Sender<RpcNotification>,
    request: RpcRequest,
    mut peer_handle: PeerHandle,
) -> Result<(), RpcError> {
    let req_data = request.raw_request;
    let request_id = request.request_id;
    let protocol_id = request.protocol_id;
    let peer_id = peer_handle.peer_id();

    trace!(
        NetworkSchema::new(&network_context).remote_peer(&peer_id),
        request_id = request_id,
        "{} Received inbound request with request_id {} and protocol_id {} from peer {}",
        network_context,
        request_id,
        protocol_id,
        peer_id.short_str(),
    );

    // Collect counters for received request.
    counters::rpc_messages(network_context, REQUEST_LABEL, RECEIVED_LABEL).inc();
    counters::rpc_bytes(network_context, REQUEST_LABEL, RECEIVED_LABEL)
        .inc_by(req_data.len() as u64);
    let timer = counters::inbound_rpc_handler_latency(network_context, protocol_id).start_timer();

    // Forward request to upper layer.
    let (res_tx, res_rx) = oneshot::channel();
    let notification = RpcNotification::RecvRpc(InboundRpcRequest {
        protocol_id,
        data: Bytes::from(req_data),
        res_tx,
    });
    notification_tx.send(notification).await?;

    // Wait for response from upper layer.
    trace!(
        NetworkSchema::new(&network_context).remote_peer(&peer_id),
        request_id = request_id,
        "{} Waiting for upstream response for inbound request with request_id {} and protocol_id {} from peer {}",
        network_context,
        request_id,
        protocol_id,
        peer_id.short_str(),
    );
    let res_data = res_rx.await??;
    let res_len = res_data.len();
    let latency = timer.stop_and_record();

    // Send response to remote peer.
    trace!(
        NetworkSchema::new(&network_context).remote_peer(&peer_id),
        request_id = request_id,
        "{} Sending response for request_id {} and protocol_id {} to peer {}. Upstream handler took {:.6} seconds.",
        network_context,
        request_id,
        protocol_id,
        peer_id.short_str(),
        latency,
    );
    let response = RpcResponse {
        raw_response: Vec::from(res_data.as_ref()),
        request_id,
        priority: request.priority,
    };
    peer_handle
        .send_message(NetworkMessage::RpcResponse(response), protocol_id)
        .await?;

    // Collect counters for sent response.
    counters::rpc_messages(network_context, RESPONSE_LABEL, SENT_LABEL).inc();
    counters::rpc_bytes(network_context, RESPONSE_LABEL, SENT_LABEL).inc_by(res_len as u64);
    Ok(())
}
