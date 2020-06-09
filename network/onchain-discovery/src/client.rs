// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    network_interface::OnchainDiscoveryNetworkSender,
    storage_query_discovery_set_async,
    types::{
        DiscoveryInfoInternal, DiscoverySetInternal, QueryDiscoverySetRequest,
        QueryDiscoverySetResponse,
    },
};
use anyhow::{Context as _, Result};
use futures::{
    future::{Future, FutureExt},
    sink::SinkExt,
    stream::{FusedStream, Stream, StreamExt},
};
use libra_config::network_id::NetworkContext;
use libra_logger::prelude::*;
use libra_types::{
    on_chain_config::ValidatorSet,
    trusted_state::{TrustedState, TrustedStateChange},
    waypoint::Waypoint,
    PeerId,
};
use network::{
    connectivity_manager::{ConnectivityRequest, DiscoverySource},
    peer_manager::{conn_notifs_channel, ConnectionNotification},
};
use option_future::OptionFuture;
use rand::{rngs::SmallRng, Rng, SeedableRng};
use std::{collections::HashSet, mem, sync::Arc, time::Duration};
use storage_interface::DbReader;

/// Actor for querying various sources (remote peers, local storage) for the
/// latest discovery set and notifying the `ConnectivityManager` of updates.
pub struct OnchainDiscovery<TTicker> {
    /// Network Context, includes all information about this node in it's network
    network_context: NetworkContext,
    /// The current trusted state, which keeps track of the latest verified
    /// ledger version and validator set. This version can be ahead of other
    /// components (e.g., state sync) since we only rely on syncing epoch change
    /// LedgerInfo's and don't need to do tx replay.
    trusted_state: TrustedState,
    /// An internal representation of the most recent discovery set.
    latest_discovery_set: DiscoverySetInternal,
    /// The set of peers we're connected to.
    connected_peers: HashSet<PeerId>,
    /// A channel to send requests to the network instance.
    network_tx: OnchainDiscoveryNetworkSender,
    /// A channel to send requests to the connectivity manager
    conn_mgr_reqs_tx: channel::Sender<ConnectivityRequest>,
    /// A channel to receive connection updates from the network.
    conn_notifs_rx: conn_notifs_channel::Receiver,
    /// internal gRPC client to send read requests to Libra Storage.
    libra_db: Arc<dyn DbReader>,
    /// Ticker to periodically query our peers for the latest discovery set.
    // TODO(philiphayes): once we do a bunch of initial queries on startup, we
    // can probably reduce the frequency.
    peer_query_ticker: TTicker,
    /// Ticker to query our own storage for the latest discovery set.
    // TODO(philiphayes): we should only need to query our own storage on startup
    // and after that rely on state-sync's reconfig notifications.
    storage_query_ticker: TTicker,
    /// The timeout for the whole rpc request when querying one of our peers for
    /// their latest discovery set.
    outbound_rpc_timeout: Duration,
    /// Random number generator. Used to sample the next peer to query.
    rng: SmallRng,
    /// A local counter incremented on receiving an incoming message. Printing this in debugging
    /// allows for easy debugging.
    event_id: u32,
}

impl<TTicker> OnchainDiscovery<TTicker>
where
    TTicker: Stream + FusedStream + Unpin,
{
    pub fn new(
        network_context: NetworkContext,
        waypoint: Waypoint,
        network_tx: OnchainDiscoveryNetworkSender,
        conn_mgr_reqs_tx: channel::Sender<ConnectivityRequest>,
        conn_notifs_rx: conn_notifs_channel::Receiver,
        libra_db: Arc<dyn DbReader>,
        peer_query_ticker: TTicker,
        storage_query_ticker: TTicker,
        outbound_rpc_timeout: Duration,
    ) -> Self {
        let trusted_state = waypoint.into();

        Self {
            network_context,
            trusted_state,
            latest_discovery_set: DiscoverySetInternal::empty(),
            connected_peers: HashSet::new(),
            network_tx,
            conn_mgr_reqs_tx,
            conn_notifs_rx,
            libra_db,
            peer_query_ticker,
            storage_query_ticker,
            outbound_rpc_timeout,
            rng: SmallRng::from_entropy(),
            event_id: 0,
        }
    }

    pub async fn start(mut self) {
        // A slot that can hold at-most-one pending outbound discovery set query
        // to another peer.
        let mut pending_outbound_rpc = OptionFuture::new(None);

        // On startup, we want to first query our local storage state to see if
        // we have an up-to-date discovery set to connect with.
        let f_query_storage = self.handle_storage_query_tick().boxed();
        let mut pending_storage_query = OptionFuture::new(Some(f_query_storage));

        debug!("{} starting onchain discovery", self.network_context);
        loop {
            self.event_id = self.event_id.wrapping_add(1);
            futures::select! {
                notif = self.conn_notifs_rx.select_next_some() => {
                    trace!("{} event id: {}, type: ConnectionNotification", self.network_context, self.event_id);
                    self.handle_connection_notif(notif);
                },
                _ = self.peer_query_ticker.select_next_some() => {
                    trace!("{} event id: {}, type: PeerQueryTick", self.network_context, self.event_id);
                    pending_outbound_rpc
                        .or_insert_with(|| {
                            self.handle_peer_query_tick()
                                .map(|f_query| f_query.boxed())
                        });
                },
                query_res = pending_outbound_rpc => {
                    trace!("{} event id: {}, type: QueryResponse", self.network_context, self.event_id);
                    self.handle_outbound_query_res(query_res).await;
                },
                _ = self.storage_query_ticker.select_next_some() => {
                    trace!("{} event id: {}, type: StorageQueryTick", self.network_context, self.event_id);
                    pending_storage_query
                        .or_insert_with(|| Some(self.handle_storage_query_tick().boxed()));
                },
                query_res = pending_storage_query => {
                    trace!("{} event id: {}, type: QueryResponse", self.network_context, self.event_id);
                    self.handle_outbound_query_res(query_res).await;
                }
                complete => {
                    crit!("{} onchain discovery terminated", self.network_context);
                    break;
                }
            }
        }
    }

    fn handle_connection_notif(&mut self, notif: ConnectionNotification) {
        match notif {
            ConnectionNotification::NewPeer(peer_id, _addr) => {
                trace!(
                    "{} connected to new peer: {}",
                    self.network_context,
                    peer_id.short_str()
                );
                // Add peer to connected peer list.
                self.connected_peers.insert(peer_id);
            }
            ConnectionNotification::LostPeer(peer_id, _addr, _reason) => {
                trace!(
                    "{} disconnected from peer: {}",
                    self.network_context,
                    peer_id.short_str()
                );
                // Remove peer from connected peer list.
                self.connected_peers.remove(&peer_id);
            }
        }
    }

    fn handle_peer_query_tick(
        &mut self,
    ) -> Option<
        impl Future<
            Output = Result<(
                PeerId,
                QueryDiscoverySetRequest,
                Box<QueryDiscoverySetResponse>,
            )>,
        >,
    > {
        self.sample_peer().map(|peer_id| {
            let req_msg = QueryDiscoverySetRequest {
                known_version: self.trusted_state.latest_version(),
            };

            let peer_id_short = peer_id.short_str();

            trace!(
                "{} handle_peer_query_tick: querying peer: {}, trusted version: {}",
                self.network_context,
                peer_id_short,
                req_msg.known_version,
            );

            peer_query_discovery_set(
                self.network_context.clone(),
                peer_id,
                req_msg,
                self.outbound_rpc_timeout,
                self.network_tx.clone(),
            )
        })
    }

    async fn handle_outbound_query_res(
        &mut self,
        query_res: Result<(
            PeerId,
            QueryDiscoverySetRequest,
            Box<QueryDiscoverySetResponse>,
        )>,
    ) {
        match query_res {
            Ok((peer_id, req_msg, res_msg)) => {
                debug!(
                    "{} received query response: peer: {}, their version: {}",
                    self.network_context,
                    peer_id.short_str(),
                    res_msg.latest_li.ledger_info().version(),
                );
                self.handle_query_response(peer_id, req_msg, res_msg).await;
            }
            Err(err) => warn!(
                "{} query to remote peer failed: {:?}",
                self.network_context, err
            ),
        }
    }

    fn handle_storage_query_tick(
        &self,
    ) -> impl Future<
        Output = Result<(
            PeerId,
            QueryDiscoverySetRequest,
            Box<QueryDiscoverySetResponse>,
        )>,
    > {
        let trusted_version = self.trusted_state.latest_version();
        trace!(
            "{} handle_storage_query_tick: querying self storage, trusted version: {}",
            self.network_context,
            trusted_version,
        );
        let req_msg = QueryDiscoverySetRequest {
            known_version: trusted_version,
        };
        let self_peer_id = self.network_context.peer_id();
        let libra_db = Arc::clone(&self.libra_db);
        storage_query_discovery_set_async(libra_db, req_msg).map(move |res| {
            // add peer_id context
            res.map(|(req_msg, res_msg)| (self_peer_id, req_msg, res_msg))
        })
    }

    async fn handle_query_response(
        &mut self,
        peer_id: PeerId,
        req_msg: QueryDiscoverySetRequest,
        res_msg: Box<QueryDiscoverySetResponse>,
    ) {
        let (trusted_state_change, opt_discovery_set) =
            match res_msg.verify_and_ratchet(&req_msg, &self.trusted_state) {
                Ok(res) => res,
                Err(err) => {
                    warn!(
                        "{} invalid query response: peer: {}, request_version: {}, err: {:?}",
                        self.network_context,
                        peer_id.short_str(),
                        req_msg.known_version,
                        err
                    );
                    return;
                }
            };

        match trusted_state_change {
            TrustedStateChange::Epoch {
                new_state,
                latest_epoch_change_li,
            } => {
                info!(
                    "{} successfully ratcheted to new epoch: \
                     peer: {}, epoch: {}, version: {}",
                    self.network_context,
                    peer_id.short_str(),
                    latest_epoch_change_li.ledger_info().epoch(),
                    new_state.latest_version(),
                );
                self.trusted_state = new_state;

                let discovery_set = opt_discovery_set.expect(
                    "We're guaranteed Some(DiscoverySet) from QueryDiscoverySetResponse::verify_and_ratchet"
                );
                self.handle_new_discovery_set_event(discovery_set).await;
            }
            TrustedStateChange::Version { new_state } => {
                debug!(
                    "{} successfully ratcheted to new version: \
                     peer: {}, version: {}",
                    self.network_context,
                    peer_id.short_str(),
                    new_state.latest_version(),
                );
                self.trusted_state = new_state;
            }
            TrustedStateChange::NoChange => (),
        };
    }

    async fn handle_new_discovery_set_event(&mut self, validator_set: ValidatorSet) {
        let latest_discovery_set =
            DiscoverySetInternal::from_validator_set(self.network_context.role(), validator_set);

        let prev_discovery_set =
            mem::replace(&mut self.latest_discovery_set, latest_discovery_set.clone());

        // TODO(philiphayes): ConnectivityManager supports multiple identity pubkeys per
        // peer (will accept connections from any in set, and do { pubkeys } x { addrs }
        // when attempting to dial).

        // Compare the latest and previous discovery sets to determine if
        // we need to update the connectivity manager that a peer is
        // advertising new addresses.
        if self.latest_discovery_set != prev_discovery_set {
            let update_addr_reqs = self
                .latest_discovery_set
                .0
                .iter()
                .map(|(peer_id, DiscoveryInfoInternal(_id_pubkey, addrs))| {
                    (*peer_id, addrs.clone())
                })
                .collect();
            self.conn_mgr_reqs_tx
                .send(ConnectivityRequest::UpdateAddresses(
                    DiscoverySource::OnChain,
                    update_addr_reqs,
                ))
                .await
                .unwrap();
        }
    }

    /// Sample iid a PeerId of one of the connected peers.
    fn sample_peer(&mut self) -> Option<PeerId> {
        let num_peers = self.connected_peers.len();
        if num_peers > 0 {
            // The particular index is not important, just the probability of
            // sampling any of the current peers is equal.
            let idx = self.rng.gen_range(0, num_peers);
            self.connected_peers.iter().nth(idx).copied()
        } else {
            None
        }
    }
}

/// Query a remote peer for their latest discovery set and a epoch change
/// proof.
async fn peer_query_discovery_set(
    network_context: NetworkContext,
    peer_id: PeerId,
    req_msg: QueryDiscoverySetRequest,
    outbound_rpc_timeout: Duration,
    mut network_tx: OnchainDiscoveryNetworkSender,
) -> Result<(
    PeerId,
    QueryDiscoverySetRequest,
    Box<QueryDiscoverySetResponse>,
)> {
    let our_latest_version = req_msg.known_version;
    let res_msg = network_tx
        .query_discovery_set(peer_id, req_msg.clone(), outbound_rpc_timeout)
        .await
        .with_context(|| {
            format!(
                "{} failed to query peer discovery set: peer: {}, latest_version: {}",
                network_context,
                peer_id.short_str(),
                our_latest_version,
            )
        })?;

    Ok((peer_id, req_msg, res_msg))
}
