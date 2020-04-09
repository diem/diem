// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

// <Black magic>
// Increase recursion limit to allow for use of select! macro.
#![recursion_limit = "1024"]
// </Black magic>

//! Protocol for discovering validator + validator full node network addresses
//! and identity keys via a shared on-chain discovery resource.
//!
//! ## Protocol
//!
//! + Bootstrapping
//!
//!     - query own storage for latest known discovery set.
//!     => peer set = seed peers (from config) and any peers in our storage's
//!        discovery set (might be very stale).
//!     - we must assume that we can connect to at least one of these peers from
//!       the bootstrapping phase, otherwise we will not be able to make progress.
//!
//! + Steady state
//!
//!     - query own storage for latest known discovery set.
//!     - query peers for their latest known discovery set and epoch changes
//!       between our current state and the peer's current state (if any).
//!     - updates to the discovery set (e.g., a validator is advertising a
//!       different network address) are sent to the connectivity manager.
//!
//! + Serving Queries
//!
//!     - all validators and validator full nodes serve queries on their current
//!       discovery set via LibraNet.
//!     - note that validators will only allow connections from other validators
//!       for DDoS mitigation.
//!     - responses include the most recent discovery set event with state proofs
//!       (emitted on reconfiguration) and any epoch changes needed to get the
//!       requestor up to the most recent epoch.
//!
//! ## Implementation
//!
//! The onchain discovery module is implemented as an actor that:
//!
//! 1. queries our own storage for our onchain discovery set
//! 2. services discovery set queries from our peers
//! 3. queries our peers for their discovery set
//! 4. notifies the connectivity_manager with updated network info whenever we
//!    detect a newer discovery set

use crate::{
    network_interface::{OnchainDiscoveryNetworkEvents, OnchainDiscoveryNetworkSender},
    types::{
        DiscoveryInfoInternal, DiscoverySetInternal, OnchainDiscoveryMsg, QueryDiscoverySetRequest,
        QueryDiscoverySetResponse, QueryDiscoverySetResponseWithEvent,
    },
};
use anyhow::{Context as AnyhowContext, Result};
use bounded_executor::BoundedExecutor;
use bytes::Bytes;
use futures::{
    channel::oneshot,
    future,
    future::{FusedFuture, Future, FutureExt},
    ready,
    stream::{FusedStream, Stream, StreamExt},
    task::Poll,
};
use libra_config::config::RoleType;
use libra_logger::prelude::*;
use libra_types::{
    discovery_set::DiscoverySetChangeEvent,
    get_with_proof::{UpdateToLatestLedgerRequest, UpdateToLatestLedgerResponse},
    trusted_state::{TrustedState, TrustedStateChange},
    waypoint::Waypoint,
    PeerId,
};
use network::{
    connectivity_manager::{ConnectivityRequest, DiscoverySource},
    error::NetworkError,
    protocols::{network::Event, rpc::error::RpcError},
};
use rand::{rngs::SmallRng, Rng, SeedableRng};
use std::{
    collections::HashSet,
    convert::{TryFrom, TryInto},
    mem,
    pin::Pin,
    sync::Arc,
    task::Context,
    time::Duration,
};
use storage_client::StorageRead;
use tokio::runtime::Handle;

#[cfg(test)]
mod test;

pub mod network_interface;
pub mod types;

pub struct OnchainDiscovery<TTicker> {
    /// A bounded executor for handling inbound discovery set queries.
    inbound_rpc_executor: BoundedExecutor,
    /// Our node's PeerId.
    peer_id: PeerId,
    /// Our node's role (validator || fullnode).
    role: RoleType,
    /// The current trusted state, which keeps track of the latest verified
    /// ledger version and validator set. This version can be ahead of other
    /// components (e.g., state sync) since we only rely on syncing epoch change
    /// LedgerInfo's and don't need to do tx replay.
    trusted_state: TrustedState,
    /// The sequence number of the most recent discovery set change event we've
    /// processed.
    latest_event_seq_num: u64,
    /// An internal representation of the most recent discovery set.
    latest_discovery_set: DiscoverySetInternal,
    /// The set of peers we're connected to.
    connected_peers: HashSet<PeerId>,
    /// A channel to send requests to the network instance.
    network_tx: OnchainDiscoveryNetworkSender,
    /// A channel to recevie notifications from the network.
    network_rx: OnchainDiscoveryNetworkEvents,
    /// internal gRPC client to send read requests to Libra Storage.
    // TODO(philiphayes): use the new storage DbReader interface.
    storage_read_client: Arc<dyn StorageRead>,
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
        executor: Handle,
        self_peer_id: PeerId,
        role: RoleType,
        // TODO(philiphayes): make Waypoint non-optional
        waypoint: Option<Waypoint>,
        network_tx: OnchainDiscoveryNetworkSender,
        network_rx: OnchainDiscoveryNetworkEvents,
        storage_read_client: Arc<dyn StorageRead>,
        peer_query_ticker: TTicker,
        storage_query_ticker: TTicker,
        outbound_rpc_timeout: Duration,
        max_concurrent_inbound_queries: usize,
    ) -> Self {
        // This initial trusted state will trust any genesis presented to us.
        // Since we are querying from our own storage, this will just get us
        // up-to-speed on our last known ledger state from storage, ratcheting
        // our trusted state in the process.
        // TODO(philiphayes): always start from at least genesis _waypoint_.
        // TODO(philiphayes): we could also start from storage.get_startup_info's
        // latest ledger info?
        // TODO(philiphayes): this is probably unsafe currently, since the
        // storage query could race with the peer query below and we might
        // accidently trust whatever the peer's genesis is.
        let trusted_state = waypoint
            .map(TrustedState::from_waypoint)
            .unwrap_or_else(TrustedState::new_trust_any_genesis_WARNING_UNSAFE);

        Self {
            inbound_rpc_executor: BoundedExecutor::new(max_concurrent_inbound_queries, executor),
            peer_id: self_peer_id,
            role,
            trusted_state,
            latest_event_seq_num: 0,
            latest_discovery_set: DiscoverySetInternal::empty(),
            connected_peers: HashSet::new(),
            network_tx,
            network_rx,
            storage_read_client,
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

        debug!("starting onchain discovery");
        loop {
            self.event_id = self.event_id.wrapping_add(1);
            futures::select! {
                event = self.network_rx.select_next_some() => {
                    trace!("event id: {}, type: NetworkEvent", self.event_id);
                    self.handle_network_event(event);
                },
                _ = self.peer_query_ticker.select_next_some() => {
                    trace!("event id: {}, type: PeerQueryTick", self.event_id);
                    pending_outbound_rpc
                        .or_insert_with(|| {
                            self.handle_peer_query_tick()
                                .map(|f_query| f_query.boxed())
                        });
                },
                query_res = pending_outbound_rpc => {
                    trace!("event id: {}, type: QueryResponse", self.event_id);
                    self.handle_outbound_query_res(query_res).await;
                },
                _ = self.storage_query_ticker.select_next_some() => {
                    trace!("event id: {}, type: StorageQueryTick", self.event_id);
                    pending_storage_query
                        .or_insert_with(|| Some(self.handle_storage_query_tick().boxed()));
                },
                query_res = pending_storage_query => {
                    trace!("event id: {}, type: QueryResponse", self.event_id);
                    self.handle_outbound_query_res(query_res).await;
                }
                complete => {
                    crit!("onchain discovery terminated");
                    break;
                }
            }
        }
    }

    fn handle_network_event(&mut self, event: Result<Event<OnchainDiscoveryMsg>, NetworkError>) {
        match event {
            Ok(event) => match event {
                Event::NewPeer(peer_id) => {
                    trace!("connected to new peer: {}", peer_id.short_str());
                    // Add peer to connected peer list.
                    self.connected_peers.insert(peer_id);
                }
                Event::LostPeer(peer_id) => {
                    trace!("disconnected from peer: {}", peer_id.short_str());
                    // Remove peer from connected peer list.
                    self.connected_peers.remove(&peer_id);
                }
                Event::Message(msg) => {
                    warn!("unexpected direct-send message from network: {:?}", msg);
                    debug_assert!(false);
                }
                Event::RpcRequest((peer_id, msg, res_tx)) => match msg.try_into() {
                    Ok(OnchainDiscoveryMsg::QueryDiscoverySetRequest(req_msg)) => {
                        debug!(
                            "recevied query discovery set request: peer: {}, \
                             version: {}, seq_num: {}",
                            peer_id.short_str(),
                            req_msg.client_known_version,
                            req_msg.client_known_seq_num,
                        );

                        let res = self.inbound_rpc_executor.try_spawn(
                            handle_query_discovery_set_request(
                                Arc::clone(&self.storage_read_client),
                                peer_id,
                                req_msg,
                                res_tx,
                            ),
                        );

                        if res.is_err() {
                            warn!(
                                "discovery set query executor at capacity; dropped \
                                 rpc request: peer: {}",
                                peer_id.short_str()
                            );
                        }
                    }
                    Ok(msg) => {
                        warn!("unexpected rpc from peer: {}, msg: {:?}", peer_id, msg);
                        debug_assert!(false);
                    }
                    Err(err) => {
                        warn!(
                            "failed to deserialize rpc from peer: {}, err: {:?}",
                            peer_id, err
                        );
                        debug_assert!(false);
                    }
                },
            },
            Err(err) => error!("network event error: {:?}", err),
        }
    }

    fn handle_peer_query_tick(
        &mut self,
    ) -> Option<
        impl Future<
            Output = Result<(
                PeerId,
                QueryDiscoverySetRequest,
                QueryDiscoverySetResponseWithEvent,
            )>,
        >,
    > {
        self.sample_peer().map(|peer_id| {
            let req_msg = QueryDiscoverySetRequest {
                client_known_version: self.trusted_state.latest_version(),
                client_known_seq_num: 0,
            };

            let peer_id_short = peer_id.short_str();

            trace!(
                "handle_peer_query_tick: querying peer: {}, trusted version: {}",
                peer_id_short,
                req_msg.client_known_version
            );

            peer_query_discovery_set(
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
            QueryDiscoverySetResponseWithEvent,
        )>,
    ) {
        match query_res {
            Ok((peer_id, req_msg, res_msg)) => {
                debug!(
                    "received query response: peer: {}, their version: {}",
                    peer_id.short_str(),
                    res_msg
                        .query_res
                        .update_to_latest_ledger_response
                        .ledger_info_with_sigs
                        .ledger_info()
                        .version(),
                );
                self.handle_query_response(peer_id, req_msg, res_msg).await;
            }
            Err(err) => warn!("query to remote peer failed: {:?}", err),
        }
    }

    fn handle_storage_query_tick(
        &self,
    ) -> impl Future<
        Output = Result<(
            PeerId,
            QueryDiscoverySetRequest,
            QueryDiscoverySetResponseWithEvent,
        )>,
    > {
        let trusted_version = self.trusted_state.latest_version();
        trace!(
            "handle_storage_query_tick: querying self storage, trusted version: {}",
            trusted_version,
        );
        let req_msg = QueryDiscoverySetRequest {
            client_known_version: trusted_version,
            client_known_seq_num: self.latest_event_seq_num,
        };
        let self_peer_id = self.peer_id;
        storage_query_discovery_set(self.storage_read_client.clone(), req_msg)
            .map(move |res| res.map(move |(req_msg, res_msg)| (self_peer_id, req_msg, res_msg)))
    }

    async fn handle_query_response(
        &mut self,
        peer_id: PeerId,
        req_msg: QueryDiscoverySetRequest,
        res_msg: QueryDiscoverySetResponseWithEvent,
    ) {
        let latest_version = self.trusted_state.latest_version();

        let trusted_state_change = match res_msg
            .query_res
            .update_to_latest_ledger_response
            .verify(&self.trusted_state, &req_msg.into())
        {
            Ok(trusted_state_change) => trusted_state_change,
            Err(err) => {
                warn!(
                    "invalid query response: failed to ratchet state: \
                     peer: {}, err: {:?}",
                    peer_id.short_str(),
                    err
                );
                return;
            }
        };

        let new_state = match trusted_state_change {
            TrustedStateChange::Epoch {
                new_state,
                latest_epoch_change_li,
                ..
            } => {
                info!(
                    "successfully ratcheted to new epoch: \
                     peer: {}, epoch: {}, version: {}",
                    peer_id.short_str(),
                    latest_epoch_change_li.ledger_info().epoch(),
                    new_state.latest_version(),
                );
                new_state
            }
            TrustedStateChange::Version { new_state, .. } => {
                debug!(
                    "successfully ratcheted to new version: \
                     peer: {}, version: {}",
                    peer_id.short_str(),
                    new_state.latest_version(),
                );
                new_state
            }
        };

        // swap in our newly ratcheted trusted state
        self.trusted_state = new_state;

        if let Some(discovery_set_event) = res_msg.event {
            self.handle_new_discovery_set_event(discovery_set_event)
                .await;
        } else {
            debug!(
                "no new discovery set event since latest version, peer: {}, latest_version: {}",
                peer_id.short_str(),
                latest_version,
            );
        }
    }

    async fn handle_new_discovery_set_event(
        &mut self,
        discovery_set_event: DiscoverySetChangeEvent,
    ) {
        let our_seq_num = self.latest_event_seq_num;
        let new_seq_num = discovery_set_event.event_seq_num;

        debug!(
            "handle_new_discovery_set_event: our_seq_num: {}, new_seq_num: {}",
            our_seq_num, new_seq_num
        );

        assert!(
            new_seq_num >= our_seq_num,
            "somehow we successfully ratcheted to a new trusted state (fresher \
             version) but the discovery set event seqnum is older! This should \
             never happen.",
        );

        // We should update if there is a newer discovery set event or we're just
        // starting up.
        let should_update = new_seq_num > our_seq_num || self.latest_discovery_set.is_empty();
        if !should_update {
            debug!("no new discovery set event; ignoring response");
            return;
        }

        // event is actually newer; update connectivity manager about any
        // modified peer infos.

        info!(
            "observed newer discovery set; updating connectivity manager: \
             our seq num: {}, new seq num: {}",
            our_seq_num, new_seq_num
        );

        let latest_discovery_set =
            DiscoverySetInternal::from_discovery_set(self.role, discovery_set_event.discovery_set);

        let mut prev_discovery_set =
            mem::replace(&mut self.latest_discovery_set, latest_discovery_set.clone());

        // TODO(philiphayes): send UpdateEligibleNodes if identity pubkeys change

        // TODO(philiphayes): ConnectivityManager supports multiple identity pubkeys per
        // peer (will accept connections from any in set, and do { pubkeys } x { addrs }
        // when attempting to dial).

        // TODO(philiphayes): consensus sends reconfig notification to on-chain
        // discovery instead of network?

        // pull out here to satisfy borrow checker
        let self_peer_id = self.peer_id;

        // Compare the latest and previous discovery sets to determine if
        // we need to update the connectivity manager that a peer is
        // advertising new addresses. In an effort to maintain
        // connectivity, we will also merge in the previous advertised
        // addresses.
        let update_addr_reqs = latest_discovery_set
            .0
            .into_iter()
            .filter(|(peer_id, _discovery_info)| &self_peer_id != peer_id)
            .filter_map(|(peer_id, DiscoveryInfoInternal(_id_pubkey, addrs))| {
                let prev_discovery_info_opt = prev_discovery_set.0.remove(&peer_id);

                let mut addrs = addrs;

                match prev_discovery_info_opt {
                    // if there's a change between prev and new discovery set,
                    // send an update request with duplicate addresses removed.
                    Some(DiscoveryInfoInternal(_prev_id_pubkey, prev_addrs))
                        if addrs != prev_addrs =>
                    {
                        for addr in prev_addrs.into_iter() {
                            if !addrs.contains(&addr) {
                                addrs.push(addr);
                            }
                        }
                        Some(ConnectivityRequest::UpdateAddresses(
                            DiscoverySource::OnChain,
                            peer_id,
                            addrs,
                        ))
                    }
                    // no change, don't send an update request
                    Some(_) => None,
                    // a validator has been added or we're starting up; always
                    // send an update request.
                    None => Some(ConnectivityRequest::UpdateAddresses(
                        DiscoverySource::OnChain,
                        peer_id,
                        addrs,
                    )),
                }
            });

        for update_addr_req in update_addr_reqs {
            self.network_tx
                .send_connectivity_request(update_addr_req)
                .await
                .unwrap();
        }

        // TODO(philiphayes): check for updated list of accepted (id key + sign key)
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

async fn handle_query_discovery_set_request(
    storage_read_client: Arc<dyn StorageRead>,
    peer_id: PeerId,
    req_msg: QueryDiscoverySetRequest,
    mut res_tx: oneshot::Sender<Result<Bytes, RpcError>>,
) {
    // TODO(philiphayes): verify that there is a timeout here...
    let mut f_rpc_cancel = future::poll_fn(|cx: &mut Context<'_>| res_tx.poll_canceled(cx)).fuse();

    // TODO(philiphayes): remove?
    let peer_id_short = peer_id.short_str();

    // cancel the internal storage rpc request early if the external rpc request
    // is cancelled.
    futures::select! {
        res = storage_query_discovery_set(storage_read_client, req_msg).fuse() => {
            let (_req_msg, res_msg) = match res {
                Ok(res) => res,
                Err(err) => {
                    warn!("error querying storage discovery set: peer: {}, err: {:?}", peer_id_short, err);
                    return;
                },
            };

            let res_msg = QueryDiscoverySetResponse::from(res_msg);
            let res_msg = OnchainDiscoveryMsg::QueryDiscoverySetResponse(res_msg);
            let res_bytes = match lcs::to_bytes(&res_msg) {
                Ok(res_bytes) => res_bytes,
                Err(err) => {
                    error!("failed to serialize response message: err: {:?}, res_msg: {:?}", err, res_msg);
                    return;
                }
            };

            if res_tx.send(Ok(res_bytes.into())).is_err() {
                debug!("remote peer canceled discovery set query: peer: {}", peer_id_short);
            }
        },
        _ = f_rpc_cancel => {
            debug!("remote peer canceled discovery set query: peer: {}", peer_id_short);
        },
    }
}

/// Query our own storage for the latest discovery set and validator change proof.
async fn storage_query_discovery_set(
    storage_read_client: Arc<dyn StorageRead>,
    req_msg: QueryDiscoverySetRequest,
) -> Result<(QueryDiscoverySetRequest, QueryDiscoverySetResponseWithEvent)> {
    let storage_req_msg = Into::<UpdateToLatestLedgerRequest>::into(req_msg.clone());

    let res_msg = storage_read_client
        .update_to_latest_ledger(
            storage_req_msg.client_known_version,
            storage_req_msg.requested_items,
        )
        .await
        .with_context(|| {
            format!(
                "error forwarding discovery query to storage: client version: {}",
                req_msg.client_known_version,
            )
        })?;

    let (response_items, ledger_info_with_sigs, validator_change_proof, ledger_consistency_proof) =
        res_msg;

    let res_msg = UpdateToLatestLedgerResponse::new(
        response_items,
        ledger_info_with_sigs,
        validator_change_proof,
        ledger_consistency_proof,
    );
    let res_msg = QueryDiscoverySetResponse::from(res_msg);
    let res_msg = QueryDiscoverySetResponseWithEvent::try_from(res_msg).with_context(|| {
        format!(
            "failed to verify storage's response: client version: {}",
            req_msg.client_known_version
        )
    })?;

    Ok((req_msg, res_msg))
}

/// Query a remote peer for their latest discovery set and a validator change
/// proof.
async fn peer_query_discovery_set(
    peer_id: PeerId,
    req_msg: QueryDiscoverySetRequest,
    outbound_rpc_timeout: Duration,
    mut network_tx: OnchainDiscoveryNetworkSender,
) -> Result<(
    PeerId,
    QueryDiscoverySetRequest,
    QueryDiscoverySetResponseWithEvent,
)> {
    let our_latest_version = req_msg.client_known_version;
    let res_msg = network_tx
        .query_discovery_set(peer_id, req_msg.clone(), outbound_rpc_timeout)
        .await
        .with_context(|| {
            format!(
                "failed to query peer discovery set: peer: {}, latest_version: {}",
                peer_id.short_str(),
                our_latest_version,
            )
        })?;

    Ok((peer_id, req_msg, res_msg))
}

/// Effectively a size=1 future queue, or a slot with at-most-one future inside.
///
/// For now, we require `F: Unpin` which mandates boxing of `async fn` futures
/// but simplifies the `OptionFuture` implementation.
// TODO(philiphayes): extract this into libra/common
pub struct OptionFuture<F> {
    inner: Option<F>,
}

impl<F> OptionFuture<F>
where
    F: Future + Unpin,
{
    pub fn new(inner: Option<F>) -> Self {
        Self { inner }
    }

    pub fn or_insert_with<G>(&mut self, fun: G)
    where
        G: FnOnce() -> Option<F>,
    {
        if self.inner.is_none() {
            self.inner = fun();
        }
    }
}

impl<F> Unpin for OptionFuture<F> where F: Unpin {}

impl<F> Future for OptionFuture<F>
where
    F: Future + Unpin,
{
    type Output = <F as Future>::Output;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        // try to poll whatever future is in the inner slot, if there is one.
        let out = match Pin::new(&mut self.as_mut().get_mut().inner).as_pin_mut() {
            Some(f) => ready!(f.poll(cx)),
            None => return Poll::Pending,
        };

        // the inner future is complete so we can drop it now and just return the
        // results.
        self.set(OptionFuture { inner: None });

        Poll::Ready(out)
    }
}

impl<F> FusedFuture for OptionFuture<F>
where
    F: Future + Unpin,
{
    fn is_terminated(&self) -> bool {
        self.inner.is_none()
    }
}
