// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use diem_types::{
    ledger_info::LedgerInfoWithSignatures,
    transaction::{TransactionListWithProof, Version},
};
use serde::{Deserialize, Serialize};
use std::fmt;

/// The response can carry different LedgerInfo types depending on whether the verification
/// is done via the local trusted validator set or a local waypoint.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum ResponseLedgerInfo {
    /// A typical response carries a LedgerInfo with signatures that should be verified using the
    /// local trusted validator set.
    /// **DEPRECATED**: `VerifiableLedgerInfo` is only required for backward compatibility. State
    /// sync avoids sending these response types and instead uses `ProgressiveLedgerInfo` below.
    /// This message will be removed on the next breaking release: https://github.com/diem/diem/issues/8013
    VerifiableLedgerInfo(LedgerInfoWithSignatures),
    /// A response to `TargetType::HighestAvailable` chunk request type.
    ProgressiveLedgerInfo {
        // LedgerInfo that the corresponding GetChunkResponse is built relative to.
        target_li: LedgerInfoWithSignatures,
        // LedgerInfo for a version later than that of `target_li`
        // If `None`, this is the same as `target_li`
        highest_li: Option<LedgerInfoWithSignatures>,
    },
    /// During the initial catchup upon startup the chunks carry LedgerInfo that is verified
    /// using the local waypoint.
    LedgerInfoForWaypoint {
        // LedgerInfo corresponding to the waypoint version.
        waypoint_li: LedgerInfoWithSignatures,
        // In case a chunk terminates an epoch, the LedgerInfo corresponding to the epoch boundary.
        end_of_epoch_li: Option<LedgerInfoWithSignatures>,
    },
}

impl ResponseLedgerInfo {
    /// The version of the LedgerInfo relative to which the transactions proofs are built.
    pub fn version(&self) -> Version {
        match self {
            ResponseLedgerInfo::VerifiableLedgerInfo(li) => li.ledger_info().version(),
            ResponseLedgerInfo::ProgressiveLedgerInfo { target_li, .. } => {
                target_li.ledger_info().version()
            }
            ResponseLedgerInfo::LedgerInfoForWaypoint { waypoint_li, .. } => {
                waypoint_li.ledger_info().version()
            }
        }
    }
}

/// The returned chunk is bounded by the end of the known_epoch of the requester
/// (i.e., a chunk never crosses epoch boundaries).
#[derive(Clone, Deserialize, Eq, PartialEq, Serialize)]
pub struct GetChunkResponse {
    /// The proofs are built relative to the LedgerInfo in `response_li`.
    /// The specifics of ledger info verification depend on its type.
    pub response_li: ResponseLedgerInfo,
    /// Chunk of transactions with proof corresponding to the ledger info carried by the response.
    pub txn_list_with_proof: TransactionListWithProof,
}

impl GetChunkResponse {
    pub fn new(
        response_li: ResponseLedgerInfo,
        txn_list_with_proof: TransactionListWithProof,
    ) -> Self {
        Self {
            response_li,
            txn_list_with_proof,
        }
    }
}

impl fmt::Debug for GetChunkResponse {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl fmt::Display for GetChunkResponse {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let txns_repr = match self.txn_list_with_proof.first_transaction_version {
            None => "empty".to_string(),
            Some(first_version) => {
                let last_version = first_version
                    .checked_add(self.txn_list_with_proof.transactions.len() as u64)
                    .and_then(|v| v.checked_sub(1)) // last_version = first_version + txns.len() - 1
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "Last version has overflown!".into());
                format!("versions [{} - {}]", first_version, last_version)
            }
        };
        let response_li_repr = match &self.response_li {
            ResponseLedgerInfo::VerifiableLedgerInfo(li) => {
                format!("[verifiable LI {}]", li.ledger_info())
            }
            ResponseLedgerInfo::ProgressiveLedgerInfo {
                target_li,
                highest_li,
            } => format!(
                "[progressive LI: target LI {}, highest LI {}]",
                target_li.ledger_info(),
                highest_li.as_ref().unwrap_or(target_li).ledger_info(),
            ),
            ResponseLedgerInfo::LedgerInfoForWaypoint {
                waypoint_li,
                end_of_epoch_li,
            } => format!(
                "[waypoint LI {}, end of epoch LI {}]",
                waypoint_li.ledger_info(),
                end_of_epoch_li
                    .as_ref()
                    .map_or("None".to_string(), |li| li.ledger_info().to_string())
            ),
        };
        write!(
            f,
            "[ChunkResponse: response li: {}, txns: {}]",
            response_li_repr, txns_repr,
        )
    }
}
