// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_types::{
    ledger_info::LedgerInfoWithSignatures,
    transaction::{TransactionListWithProof, Version},
};
use serde::{Deserialize, Serialize};
use std::fmt;

/// The response can carry different LedgerInfo types depending on whether the verification
/// is done via the local trusted validator set or a local waypoint.
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub enum ResponseLedgerInfo {
    /// A typical response carries a LedgerInfo with signatures that should be verified using the
    /// local trusted validator set.
    VerifiableLedgerInfo(LedgerInfoWithSignatures),
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
            ResponseLedgerInfo::LedgerInfoForWaypoint { waypoint_li, .. } => {
                waypoint_li.ledger_info().version()
            }
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
/// The returned chunk is bounded by the end of the known_epoch of the requester
/// (i.e., a chunk never crosses epoch boundaries).
pub struct GetChunkResponse {
    /// The proofs are built relative to the LedgerInfo in `response_ledger_info`.
    /// The specifics of ledger info verification depend on its type.
    pub response_li: ResponseLedgerInfo,
    /// chunk of transactions with proof corresponding to the ledger info carried by the response.
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

impl fmt::Display for GetChunkResponse {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut txns_list = "".to_owned();
        for txn in &self.txn_list_with_proof.transactions {
            if let Ok(user_txn) = txn.as_signed_user_txn() {
                txns_list = txns_list + &format!("{}:{} ", user_txn.sender(), user_txn.sequence_number());
            }
        }
        let txns_repr = match self.txn_list_with_proof.first_transaction_version {
            None => "empty".to_string(),
            Some(first_ver) => format!(
                "versions [{} - {}]",
                first_ver,
                first_ver - 1 + self.txn_list_with_proof.len() as u64
            ),
        };
        let response_li_repr = match &self.response_li {
            ResponseLedgerInfo::VerifiableLedgerInfo(li) => {
                format!("[verifiable LI {}]", li.ledger_info())
            }
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
            "[ChunkResponse: response li: {}, txns: {}, list: {}]",
            response_li_repr, txns_repr, txns_list,
        )
    }
}
