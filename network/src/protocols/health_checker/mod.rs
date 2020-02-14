// Copyright (c) The Libra Core Contributors
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
    proto::{HealthCheckerMsg, HealthCheckerMsg_oneof, Ping, Pong},
    protocols::rpc::error::RpcError,
    utils::MessageExt,
    validator_network::{Event, HealthCheckerNetworkEvents, HealthCheckerNetworkSender},
};
use bytes::Bytes;
use futures::{
    channel::oneshot,
    stream::{FusedStream, FuturesUnordered, Stream, StreamExt},
};
use libra_logger::prelude::*;
use libra_types::PeerId;
use rand::{rngs::SmallRng, seq::SliceRandom, SeedableRng, Rng};
use std::{collections::HashMap, time::Duration};

#[cfg(test)]
mod test;

/// The actor performing health checks by running the Ping protocol
pub struct HealthChecker<TTicker> {
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
        ticker: TTicker,
        network_tx: HealthCheckerNetworkSender,
        network_rx: HealthCheckerNetworkEvents,
        ping_timeout: Duration,
        ping_failures_tolerated: u64,
    ) -> Self {
        HealthChecker {
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
        loop {
            futures::select! {
                event = self.network_rx.select_next_some() => {
                    match event {
                        Ok(Event::NewPeer(peer_id)) => {
                            self.connected.insert(peer_id, (self.round, 0));
                        },
                        Ok(Event::LostPeer(peer_id)) => {
                            self.connected.remove(&peer_id);
                        },
                        Ok(Event::RpcRequest((peer_id, msg, res_tx))) => {
                            if let Some(HealthCheckerMsg_oneof::Ping(ping_msg)) = msg.message {
                                self.handle_ping_request(peer_id, ping_msg, res_tx);
                            } else {
                                security_log(SecurityEvent::InvalidHealthCheckerMsg)
                                    .error("Unexpected rpc message")
                                    .data(&msg)
                                    .data(&peer_id)
                                    .log();
                                debug_assert!(false, "Unexpected rpc message");
                            }
                        }
                        Ok(Event::Message(_)) => {
                            security_log(SecurityEvent::InvalidNetworkEventHC)
                                .error("Unexpected network event")
                                .data(&event)
                                .log();
                            debug_assert!(false, "Unexpected network event");
                        },
                        Err(err) => {
                            security_log(SecurityEvent::InvalidNetworkEventHC)
                                .error(&err)
                                .log();
                            debug_assert!(false, "Unexpected network error");
                        }
                    }
                }
                _ = self.ticker.select_next_some() => {
                    self.round += 1;
                    debug!("Tick: Round number: {}", self.round);
                    match self.sample_random_peer() {
                        Some(peer_id) => {
                            debug!("Will ping: {}", peer_id.short_str());

                            let nonce = self.sample_nonce();

                            tick_handlers.push(
                                Self::ping_peer(
                                    self.network_tx.clone(),
                                    peer_id,
                                    self.round,
                                    nonce,
                                    self.ping_timeout.clone()));
                        }
                        None => {
                            debug!("No connected peer to ping");
                        }
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
        crit!("Health checker actor terminated");
    }

    fn handle_ping_request(
        &mut self,
        peer_id: PeerId,
        ping_msg: Ping,
        res_tx: oneshot::Sender<Result<Bytes, RpcError>>,
    ) {
        let nonce = ping_msg.nonce;
        let pong_msg = Pong { nonce };
        let res_msg = HealthCheckerMsg {
            message: Some(HealthCheckerMsg_oneof::Pong(pong_msg)),
        };
        debug!(
            "Sending Pong response to peer: {} with nonce: {}",
            peer_id.short_str(),
            nonce
        );
        let res_data = res_msg.to_bytes().unwrap();
        let _ = res_tx.send(Ok(res_data));
    }

    async fn handle_ping_response(
        &mut self,
        peer_id: PeerId,
        round: u64,
        req_nonce: u32,
        ping_result: Result<Pong, RpcError>,
    ) {
        debug!("Got result for ping round: {}", round);
        match ping_result {
            Ok(pong_msg) => {
                let res_nonce = pong_msg.nonce;
                if res_nonce == req_nonce {
                    debug!("Ping successful for peer: {}", peer_id.short_str());
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
                    security_log(SecurityEvent::InvalidHealthCheckerMsg)
                        .error("Pong nonce doesn't match our challenge Ping nonce")
                        .data(&peer_id)
                        .data(req_nonce)
                        .data(&pong_msg)
                        .log();
                    debug_assert!(false, "Pong nonce doesn't match our challenge Ping nonce");
                }
            }
            Err(err) => {
                warn!(
                    "Ping failed for peer: {} with error: {:?}",
                    peer_id.short_str(),
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
                            info!("Disonnecting from peer: {}", peer_id.short_str());
                            if let Err(err) = self.network_tx.disconnect_peer(peer_id).await {
                                warn!(
                                    "Failed to disconnect from peer: {} with error: {:?}",
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
        mut network_tx: HealthCheckerNetworkSender,
        peer_id: PeerId,
        round: u64,
        nonce: u32,
        ping_timeout: Duration,
    ) -> (PeerId, u64, u32, Result<Pong, RpcError>) {
        let ping_msg = Ping { nonce };
        debug!(
            "Sending Ping request to peer: {} with nonce: {}",
            peer_id.short_str(),
            nonce
        );
        let res_pong_msg = network_tx.ping(peer_id, ping_msg, ping_timeout).await;
        (peer_id, round, nonce, res_pong_msg)
    }

    fn sample_random_peer(&mut self) -> Option<PeerId> {
        let peers: Vec<_> = self.connected.keys().cloned().collect();
        peers.choose(&mut self.rng).cloned()
    }

    fn sample_nonce(&mut self) -> u32 {
        self.rng.gen::<u32>()
    }
}
