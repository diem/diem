// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Protocol used to ensure peer liveness
//!
//! The HealthChecker is responsible for ensuring liveness of all peers of a node.
//! It does so by periodically selecting a random connected peer and sending a Ping probe. A
//! healthy peer is expected to respond with a corresponding Pong message.
//!
//! If a certain number of successive liveness probes for a peer fail, the HealthChecker initiates a
//! disconnect from the peer. It relies on ConnectivityManager or the remote peer to re-establish
//! the connection.
//!
//! Future Work
//! -----------
//! We can make a few other improvements to the health checker. These are:
//! - Make the policy for interpreting ping failures pluggable
//! - Use successful inbound pings as a sign of remote note being healthy
//! - Ping a peer only in periods of no application-level communication with the peer
use crate::{
    constants::NETWORK_CHANNEL_SIZE,
    counters,
    error::NetworkError,
    logging::NetworkSchema,
    peer_manager::{ConnectionRequestSender, PeerManagerRequestSender},
    protocols::{
        network::{Event, NetworkEvents, NetworkSender, NewNetworkSender},
        rpc::error::RpcError,
    },
    ProtocolId,
};
use bytes::Bytes;
use channel::message_queues::QueueStyle;
use diem_config::network_id::NetworkContext;
use diem_logger::prelude::*;
use diem_metrics::IntCounterVec;
use diem_types::PeerId;
use futures::{
    channel::oneshot,
    stream::{FusedStream, FuturesUnordered, Stream, StreamExt},
};
use rand::{rngs::SmallRng, Rng, SeedableRng};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc, time::Duration};

pub mod builder;
#[cfg(test)]
mod test;

/// The interface from Network to HealthChecker layer.
///
/// `HealthCheckerNetworkEvents` is a `Stream` of `PeerManagerNotification` where the
/// raw `Bytes` rpc messages are deserialized into
/// `HealthCheckerMsg` types. `HealthCheckerNetworkEvents` is a thin wrapper
/// around an `channel::Receiver<PeerManagerNotification>`.
pub type HealthCheckerNetworkEvents = NetworkEvents<HealthCheckerMsg>;

/// The interface from HealthChecker to Networking layer.
///
/// This is a thin wrapper around a `NetworkSender<HealthCheckerMsg>`, so it is
/// easy to clone and send off to a separate task. For example, the rpc requests
/// return Futures that encapsulate the whole flow, from sending the request to
/// remote, to finally receiving the response and deserializing. It therefore
/// makes the most sense to make the rpc call on a separate async task, which
/// requires the `HealthCheckerNetworkSender` to be `Clone` and `Send`.
#[derive(Clone)]
pub struct HealthCheckerNetworkSender {
    inner: NetworkSender<HealthCheckerMsg>,
}

/// Configuration for the network endpoints to support HealthChecker.
pub fn network_endpoint_config() -> (
    Vec<ProtocolId>,
    Vec<ProtocolId>,
    QueueStyle,
    usize,
    Option<&'static IntCounterVec>,
) {
    (
        vec![ProtocolId::HealthCheckerRpc],
        vec![],
        QueueStyle::LIFO,
        NETWORK_CHANNEL_SIZE,
        Some(&counters::PENDING_HEALTH_CHECKER_NETWORK_EVENTS),
    )
}

impl NewNetworkSender for HealthCheckerNetworkSender {
    fn new(
        peer_mgr_reqs_tx: PeerManagerRequestSender,
        connection_reqs_tx: ConnectionRequestSender,
    ) -> Self {
        Self {
            inner: NetworkSender::new(peer_mgr_reqs_tx, connection_reqs_tx),
        }
    }
}

impl HealthCheckerNetworkSender {
    /// Send a HealthChecker Ping RPC request to remote peer `recipient`. Returns
    /// the remote peer's future `Pong` reply.
    ///
    /// The rpc request can be canceled at any point by dropping the returned
    /// future.
    pub async fn send_rpc(
        &mut self,
        recipient: PeerId,
        req_msg: HealthCheckerMsg,
        timeout: Duration,
    ) -> Result<HealthCheckerMsg, RpcError> {
        let protocol = ProtocolId::HealthCheckerRpc;
        self.inner
            .send_rpc(recipient, protocol, req_msg, timeout)
            .await
    }

    pub async fn disconnect_peer(&mut self, peer_id: PeerId) -> Result<(), NetworkError> {
        self.inner.disconnect_peer(peer_id).await
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum HealthCheckerMsg {
    Ping(Ping),
    Pong(Pong),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Ping(u32);

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Pong(u32);

/// The actor performing health checks by running the Ping protocol
pub struct HealthChecker<TTicker> {
    network_context: Arc<NetworkContext>,
    /// Ticker to trigger ping to a random peer. In production, the ticker is likely to be
    /// fixed duration interval timer.
    ticker: TTicker,
    /// Channel to send requests to Network layer.
    network_tx: HealthCheckerNetworkSender,
    /// Channel to receive notifications from Network layer about new/lost connections.
    network_rx: HealthCheckerNetworkEvents,
    /// Map from connected peer to last round of successful ping, and number of failures since
    /// then.
    connected: HashMap<PeerId, (u64, u64)>,
    /// Random-number generator.
    rng: SmallRng,
    /// Ping timmeout duration.
    ping_timeout: Duration,
    /// Number of successive ping failures we tolerate before declaring a node as unhealthy and
    /// disconnecting from it. In the future, this can be replaced with a more general failure
    /// detection policy.
    ping_failures_tolerated: u64,
    /// Counter incremented in each round of health checks
    round: u64,
}

impl<TTicker> HealthChecker<TTicker>
where
    TTicker: Stream + FusedStream + Unpin,
{
    /// Create new instance of the [`HealthChecker`] actor.
    pub fn new(
        network_context: Arc<NetworkContext>,
        ticker: TTicker,
        network_tx: HealthCheckerNetworkSender,
        network_rx: HealthCheckerNetworkEvents,
        ping_timeout: Duration,
        ping_failures_tolerated: u64,
    ) -> Self {
        HealthChecker {
            network_context,
            ticker,
            network_tx,
            network_rx,
            connected: HashMap::new(),
            rng: SmallRng::from_entropy(),
            ping_timeout,
            ping_failures_tolerated,
            round: 0,
        }
    }

    pub async fn start(mut self) {
        let mut tick_handlers = FuturesUnordered::new();
        info!(
            NetworkSchema::new(&self.network_context),
            "{} Health checker actor started", self.network_context
        );
        loop {
            futures::select! {
                event = self.network_rx.select_next_some() => {
                    match event {
                        Event::NewPeer(peer_id, _origin) => {
                            self.connected.insert(peer_id, (self.round, 0));
                        }
                        Event::LostPeer(peer_id, _origin) => {
                            self.connected.remove(&peer_id);
                        }
                        Event::RpcRequest(peer_id, msg, res_tx) => {
                            match msg {
                                HealthCheckerMsg::Ping(ping) => self.handle_ping_request(peer_id, ping, res_tx),
                                _ => {
                                    warn!(
                                        SecurityEvent::InvalidHealthCheckerMsg,
                                        NetworkSchema::new(&self.network_context).remote_peer(&peer_id),
                                        rpc_message = msg,
                                        "{} Unexpected RPC message from {}",
                                        self.network_context,
                                        peer_id
                                    );
                                }
                            };
                        }
                        Event::Message(peer_id, msg) => {
                            error!(
                                SecurityEvent::InvalidNetworkEventHC,
                                NetworkSchema::new(&self.network_context).remote_peer(&peer_id),
                                "{} Unexpected direct send from {} msg {:?}",
                                self.network_context,
                                peer_id,
                                msg,
                            );
                            debug_assert!(false, "Unexpected network event");
                        }
                    }
                }
                _ = self.ticker.select_next_some() => {
                    self.round += 1;

                    if self.connected.is_empty() {
                        trace!(
                            NetworkSchema::new(&self.network_context),
                            round = self.round,
                            "{} No connected peer to ping round: {}",
                            self.network_context,
                            self.round
                        );
                        continue
                    }

                    let peers: Vec<_> = self.connected.keys().cloned().collect();
                    for peer_id in peers {
                        let nonce = self.nonce();
                        trace!(
                            NetworkSchema::new(&self.network_context),
                            round = self.round,
                            "{} Will ping: {} for round: {} nonce: {}",
                            self.network_context,
                            peer_id.short_str(),
                            self.round,
                            nonce
                        );

                        tick_handlers.push(Self::ping_peer(
                            self.network_context.clone(),
                            self.network_tx.clone(),
                            peer_id,
                            self.round,
                            nonce,
                            self.ping_timeout,
                        ));
                    }
                }
                res = tick_handlers.select_next_some() => {
                    let (peer_id, round, nonce, ping_result) = res;
                    self.handle_ping_response(peer_id, round, nonce, ping_result).await;
                }
                complete => {
                    break;
                }
            }
        }
        warn!(
            NetworkSchema::new(&self.network_context),
            "{} Health checker actor terminated", self.network_context
        );
    }

    fn handle_ping_request(
        &mut self,
        peer_id: PeerId,
        ping: Ping,
        res_tx: oneshot::Sender<Result<Bytes, RpcError>>,
    ) {
        let message = match bcs::to_bytes(&HealthCheckerMsg::Pong(Pong(ping.0))) {
            Ok(msg) => msg,
            Err(e) => {
                warn!(
                    NetworkSchema::new(&self.network_context),
                    error = ?e,
                    "{} Unable to serialize pong response: {}", self.network_context, e
                );
                return;
            }
        };
        trace!(
            NetworkSchema::new(&self.network_context).remote_peer(&peer_id),
            "{} Sending Pong response to peer: {} with nonce: {}",
            self.network_context,
            peer_id.short_str(),
            ping.0,
        );
        let _ = res_tx.send(Ok(message.into()));
    }

    async fn handle_ping_response(
        &mut self,
        peer_id: PeerId,
        round: u64,
        req_nonce: u32,
        ping_result: Result<Pong, RpcError>,
    ) {
        match ping_result {
            Ok(pong) => {
                if pong.0 == req_nonce {
                    trace!(
                        NetworkSchema::new(&self.network_context).remote_peer(&peer_id),
                        rount = round,
                        "{} Ping successful for peer: {} round: {}",
                        self.network_context,
                        peer_id.short_str(),
                        round
                    );
                    // Update last successful ping to current round.
                    self.connected
                        .entry(peer_id)
                        .and_modify(|(ref mut r, ref mut count)| {
                            if round > *r {
                                *r = round;
                                *count = 0;
                            }
                        });
                } else {
                    warn!(
                        SecurityEvent::InvalidHealthCheckerMsg,
                        NetworkSchema::new(&self.network_context).remote_peer(&peer_id),
                        "{} Pong nonce doesn't match Ping nonce. Round: {}, Pong: {}, Ping: {}",
                        self.network_context,
                        round,
                        pong.0,
                        req_nonce
                    );
                    debug_assert!(false, "Pong nonce doesn't match our challenge Ping nonce");
                }
            }
            Err(err) => {
                warn!(
                    NetworkSchema::new(&self.network_context)
                        .remote_peer(&peer_id),
                    error = ?err,
                    round = round,
                    "{} Ping failed for peer: {} round: {} with error: {:?}",
                    self.network_context,
                    peer_id.short_str(),
                    round,
                    err
                );
                match self.connected.get_mut(&peer_id) {
                    None => {
                        // If we are no longer connected to the peer, we ignore ping
                        // failure.
                    }
                    Some((ref mut prev, ref mut failures)) => {
                        // If this is the result of an older ping, we ignore it.
                        if *prev > round {
                            return;
                        }
                        // Increment num of failures. If the ping failures are now more than
                        // `self.ping_failures_tolerated`, we disconnect from the node.
                        // The HealthChecker only performs the disconnect. It relies on
                        // ConnectivityManager or the remote peer to re-establish the connection.
                        *failures += 1;
                        if *failures > self.ping_failures_tolerated {
                            info!(
                                NetworkSchema::new(&self.network_context).remote_peer(&peer_id),
                                "{} Disconnecting from peer: {}",
                                self.network_context,
                                peer_id.short_str()
                            );
                            if let Err(err) = self.network_tx.disconnect_peer(peer_id).await {
                                warn!(
                                    NetworkSchema::new(&self.network_context)
                                        .remote_peer(&peer_id),
                                    error = ?err,
                                    "{} Failed to disconnect from peer: {} with error: {:?}",
                                    self.network_context,
                                    peer_id.short_str(),
                                    err
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    async fn ping_peer(
        network_context: Arc<NetworkContext>,
        mut network_tx: HealthCheckerNetworkSender,
        peer_id: PeerId,
        round: u64,
        nonce: u32,
        ping_timeout: Duration,
    ) -> (PeerId, u64, u32, Result<Pong, RpcError>) {
        trace!(
            NetworkSchema::new(&network_context).remote_peer(&peer_id),
            round = round,
            "{} Sending Ping request to peer: {} for round: {} nonce: {}",
            network_context,
            peer_id.short_str(),
            round,
            nonce
        );
        let res_pong_msg = network_tx
            .send_rpc(peer_id, HealthCheckerMsg::Ping(Ping(nonce)), ping_timeout)
            .await
            .and_then(|msg| match msg {
                HealthCheckerMsg::Pong(res) => Ok(res),
                _ => Err(RpcError::InvalidRpcResponse),
            });
        (peer_id, round, nonce, res_pong_msg)
    }

    fn nonce(&mut self) -> u32 {
        self.rng.gen::<u32>()
    }
}
