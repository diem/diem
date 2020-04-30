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
use futures::{
    future::{FusedFuture, Future},
    ready,
    task::Poll,
};
use libra_types::get_with_proof::{UpdateToLatestLedgerRequest, UpdateToLatestLedgerResponse};
use std::{convert::TryFrom, pin::Pin, sync::Arc, task::Context};
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
