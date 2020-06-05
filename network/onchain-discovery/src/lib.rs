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
    client::OnchainDiscovery,
    service::OnchainDiscoveryService,
    types::{QueryDiscoverySetRequest, QueryDiscoverySetResponse},
};
use anyhow::{Context as AnyhowContext, Result};
use futures::{
    future::{Future, FutureExt},
    stream::StreamExt,
};
use libra_config::config::RoleType;
use libra_types::{account_config, waypoint::Waypoint, PeerId};
use network::validator_network::network_builder::NetworkBuilder;
use std::{sync::Arc, time::Duration};
use storage_interface::DbReader;
use tokio::{runtime::Handle, task, time::interval};

#[cfg(test)]
mod test;

pub mod client;
pub mod network_interface;
pub mod service;
pub mod types;

/// Query our own storage for the latest discovery set and epoch change proof.
fn storage_query_discovery_set(
    libra_db: Arc<dyn DbReader>,
    req_msg: QueryDiscoverySetRequest,
) -> Result<(QueryDiscoverySetRequest, Box<QueryDiscoverySetResponse>)> {
    // TODO(philiphayes): how to deal with partial epoch change proof?
    let (latest_li, epoch_change_proof, accumulator_proof) = libra_db
        .get_state_proof(req_msg.known_version)
        .with_context(|| {
            format!(
                "error getting state proof from storage: request version: {}",
                req_msg.known_version,
            )
        })?;

    // Only return the discovery set account if the requestor is not in the most
    // recent epoch.
    let account_state = if epoch_change_proof.ledger_info_with_sigs.is_empty() {
        None
    } else {
        Some(
            libra_db.get_account_state_with_proof(
                account_config::validator_set_address(), req_msg.known_version, latest_li.ledger_info().version()
            ).with_context(|| {
                format!("error getting discovery account state with proof from storage: request version: {}, ledger info version: {}",
                    req_msg.known_version, latest_li.ledger_info().version(),
                )
            })?
        )
    };

    let res_msg = Box::new(QueryDiscoverySetResponse {
        latest_li,
        epoch_change_proof,
        accumulator_proof,
        account_state,
    });

    Ok((req_msg, res_msg))
}

/// Query storage but async and wrapped in a [`task::spawn_blocking`](tokio::task::spawn_blocking),
/// which runs the task on a special-purpose threadpool for blocking tasks.
///
/// If you are querying in an async context, it's preferable to use this method
/// over the blocking `storage_query_discovery_set`.
fn storage_query_discovery_set_async(
    libra_db: Arc<dyn DbReader>,
    req_msg: QueryDiscoverySetRequest,
) -> impl Future<Output = Result<(QueryDiscoverySetRequest, Box<QueryDiscoverySetResponse>)>> {
    task::spawn_blocking(move || storage_query_discovery_set(libra_db, req_msg)).map(|res| {
        // flatten errors
        res.map_err(anyhow::Error::from).and_then(|res| res)
    })
}

pub fn setup_onchain_discovery(
    network: &mut NetworkBuilder,
    peer_id: PeerId,
    role: RoleType,
    libra_db: Arc<dyn DbReader>,
    waypoint: Waypoint,
    executor: &Handle,
) {
    let (network_tx, discovery_events) = network_interface::add_to_network(network);
    let outbound_rpc_timeout = Duration::from_secs(30);
    let max_concurrent_inbound_queries = 8;
    let (peer_mgr_notifs_rx, conn_notifs_rx) = (
        discovery_events.peer_mgr_notifs_rx,
        discovery_events.connection_notifs_rx,
    );

    let onchain_discovery_service = OnchainDiscoveryService::new(
        executor.clone(),
        peer_mgr_notifs_rx,
        Arc::clone(&libra_db),
        max_concurrent_inbound_queries,
    );
    executor.spawn(onchain_discovery_service.start());

    let onchain_discovery = executor.enter(move || {
        let peer_query_ticker = interval(Duration::from_secs(30)).fuse();
        let storage_query_ticker = interval(Duration::from_secs(30)).fuse();

        OnchainDiscovery::new(
            peer_id,
            role,
            waypoint,
            network_tx,
            network
                .conn_mgr_reqs_tx()
                .expect("ConnecitivtyManager not enabled"),
            conn_notifs_rx,
            libra_db,
            peer_query_ticker,
            storage_query_ticker,
            outbound_rpc_timeout,
        )
    });
    executor.spawn(onchain_discovery.start());
}
