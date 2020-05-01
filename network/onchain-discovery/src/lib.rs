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

use crate::types::{
    QueryDiscoverySetRequest, QueryDiscoverySetResponse, QueryDiscoverySetResponseWithEvent,
};
use anyhow::{Context as AnyhowContext, Result};
use libra_types::get_with_proof::{UpdateToLatestLedgerRequest, UpdateToLatestLedgerResponse};
use std::{convert::TryFrom, sync::Arc};
use storage_client::StorageRead;

#[cfg(test)]
mod test;

pub mod client;
pub mod network_interface;
pub mod service;
pub mod types;

/// Query our own storage for the latest discovery set and epoch change proof.
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

    let (response_items, ledger_info_with_sigs, epoch_change_proof, ledger_consistency_proof) =
        res_msg;

    let res_msg = UpdateToLatestLedgerResponse::new(
        response_items,
        ledger_info_with_sigs,
        epoch_change_proof,
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
