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
    error::NetworkError,
    peer_manager::{PeerManagerNotification, PeerManagerRequestSender},
    proto::{Ping, Pong},
    utils::{read_proto, MessageExt},
    ProtocolId,
};
use channel;
use futures::{
    future::{FutureExt, TryFutureExt},
    io::{AsyncRead, AsyncWrite},
    sink::SinkExt,
    stream::{FusedStream, FuturesUnordered, Stream, StreamExt},
};
use libra_logger::prelude::*;
use libra_types::PeerId;
use netcore::compat::IoCompat;
use rand::{rngs::SmallRng, seq::SliceRandom, FromEntropy};
use std::{collections::HashMap, fmt::Debug, time::Duration};
use tokio::{
    codec::{Framed, LengthDelimitedCodec},
    future::FutureExt as _,
};

#[cfg(test)]
mod test;

/// Protocol name for Ping.
pub const PING_PROTOCOL_NAME: &[u8] = b"/libra/ping/0.1.0";

/// The actor performing health checks by running the Ping protocol
pub struct HealthChecker<TTicker, TSubstream> {
    /// Ticker to trigger ping to a random peer. In production, the ticker is likely to be
    /// fixed duration interval timer.
    ticker: TTicker,
    /// Channel to send requests to PeerManager.
    peer_mgr_reqs_tx: PeerManagerRequestSender<TSubstream>,
    /// Channel to receive notifications from PeerManager about new/lost connections.
    peer_mgr_notifs_rx: channel::Receiver<PeerManagerNotification<TSubstream>>,
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

impl<TTicker, TSubstream> HealthChecker<TTicker, TSubstream>
where
    TTicker: Stream + FusedStream + Unpin,
    TSubstream: AsyncRead + AsyncWrite + Send + Unpin + Debug + 'static,
{
    /// Create new instance of the [`HealthChecker`] actor.
    pub fn new(
        ticker: TTicker,
        peer_mgr_reqs_tx: PeerManagerRequestSender<TSubstream>,
        peer_mgr_notifs_rx: channel::Receiver<PeerManagerNotification<TSubstream>>,
        ping_timeout: Duration,
        ping_failures_tolerated: u64,
    ) -> Self {
        HealthChecker {
            ticker,
            peer_mgr_reqs_tx,
            peer_mgr_notifs_rx,
            connected: HashMap::new(),
            rng: SmallRng::from_entropy(),
            ping_timeout,
            ping_failures_tolerated,
            round: 0,
        }
    }

    pub async fn start(mut self) {
        let mut tick_handlers = FuturesUnordered::new();
        let mut ping_handlers = FuturesUnordered::new();
        loop {
            futures::select! {
                notif = self.peer_mgr_notifs_rx.select_next_some() => {
                    match notif {
                        PeerManagerNotification::NewPeer(peer_id, _) => {
                            self.connected.insert(peer_id, (self.round, 0));
                        }
                        PeerManagerNotification::LostPeer(peer_id, _) => {
                            self.connected.remove(&peer_id);
                        }
                        PeerManagerNotification::NewInboundSubstream(peer_id, substream) => {
                            assert_eq!(substream.protocol, PING_PROTOCOL_NAME);
                            ping_handlers.push(Self::handle_ping(peer_id, substream.substream));
                        }
                    }
                }
                _ = self.ticker.select_next_some() => {
                    self.round += 1;
                    debug!("Round number: {}", self.round);
                    match self.get_random_peer() {
                        Some(peer_id) => {
                            debug!("Will ping: {}", peer_id.short_str());
                            tick_handlers.push(
                                Self::ping_peer(
                                    peer_id,
                                    self.round,
                                    self.peer_mgr_reqs_tx.clone(),
                                    self.ping_timeout.clone()));
                        }
                        None => {
                            debug!("No connected peer to ping");
                        }
                    }
                }
                res = tick_handlers.select_next_some() => {
                    let (peer_id, round, ping_result) = res;
                    self.handle_ping_result(peer_id, round, ping_result).await;
                }
                _ = ping_handlers.select_next_some() => {}
                complete => {
                    crit!("Health checker actor terminated");
                    break;
                }
            }
        }
    }

    async fn handle_ping_result(
        &mut self,
        peer_id: PeerId,
        round: u64,
        ping_result: Result<(), NetworkError>,
    ) {
        debug!("Got result for ping round: {}", round);
        match ping_result {
            Ok(_) => {
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
                            if let Err(err) = self.peer_mgr_reqs_tx.disconnect_peer(peer_id).await {
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
        peer_id: PeerId,
        round: u64,
        mut peer_mgr_reqs_tx: PeerManagerRequestSender<TSubstream>,
        ping_timeout: Duration,
    ) -> (PeerId, u64, Result<(), NetworkError>) {
        let ping_result = async move {
            // Request a new substream to peer.
            debug!(
                "Opening a new substream with peer: {} for Ping",
                peer_id.short_str()
            );
            let substream = peer_mgr_reqs_tx
                .open_substream(peer_id, ProtocolId::from_static(PING_PROTOCOL_NAME))
                .await?;
            // Messages are length-prefixed. Wrap in a framed stream.
            let mut substream = Framed::new(IoCompat::new(substream), LengthDelimitedCodec::new());
            // Send Ping.
            debug!("Sending Ping to peer: {}", peer_id.short_str());
            substream
                .send(
                    Ping::default()
                        .to_bytes()
                        .expect("Protobuf serialization fails"),
                )
                .await?;
            // Read Pong.
            debug!("Waiting for Pong from peer: {}", peer_id.short_str());
            let _: Pong = read_proto(&mut substream).await?;
            // Return success.
            Ok(())
        };
        (
            peer_id,
            round,
            ping_result
                .timeout(ping_timeout)
                .map_err(Into::<NetworkError>::into)
                .map(|r| r.and_then(|x| x))
                .await,
        )
    }

    async fn handle_ping(peer_id: PeerId, substream: TSubstream) {
        // Messages are length-prefixed. Wrap in a framed stream.
        let mut substream = Framed::new(IoCompat::new(substream), LengthDelimitedCodec::new());
        // Read ping.
        trace!("Waiting for Ping on new substream");
        let maybe_ping: Result<Ping, NetworkError> = read_proto(&mut substream).await;
        if let Err(err) = maybe_ping {
            warn!(
                "Failed to read ping from peer: {}. Error: {:?}",
                peer_id.short_str(),
                err
            );
            return;
        }
        // Send Pong.
        trace!("Sending Pong back");
        if let Err(err) = substream
            .send(
                Pong::default()
                    .to_bytes()
                    .expect("Protobuf serialization fails"),
            )
            .await
        {
            warn!(
                "Failed to send pong to peer: {}. Error: {:?}",
                peer_id.short_str(),
                err
            );
            return;
        }
    }

    fn get_random_peer(&mut self) -> Option<PeerId> {
        let peers: Vec<_> = self.connected.keys().cloned().collect();
        peers.choose(&mut self.rng).cloned()
    }
}
