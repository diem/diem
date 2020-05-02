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

use crate::types::{QueryDiscoverySetRequest, QueryDiscoverySetResponse};
use anyhow::{Context as AnyhowContext, Result};
use libra_types::account_config;
use std::sync::Arc;
use storage_interface::DbReader;

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
                account_config::discovery_set_address(), req_msg.known_version, latest_li.ledger_info().version()
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
