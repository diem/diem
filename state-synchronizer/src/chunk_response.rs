// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{Error, Result};
use libra_types::crypto_proxies::LedgerInfoWithSignatures;
use libra_types::transaction::TransactionListWithProof;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::fmt;

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
/// The returned chunk is bounded by the end of the known_epoch of the requester
/// (i.e., a chunk never crosses epoch boundaries).
pub struct GetChunkResponse {
    /// Transaction proofs are built relative to this ledger info.
    /// It could either be the end of the epoch of the known version in the request,
    /// or a target / highest LI.
    pub ledger_info_with_sigs: LedgerInfoWithSignatures,
    /// chunk of transactions with proof corresponding to version in `ledger_info_with_sigs`
    pub txn_list_with_proof: TransactionListWithProof,
}

impl GetChunkResponse {
    pub fn new(
        ledger_info_with_sigs: LedgerInfoWithSignatures,
        txn_list_with_proof: TransactionListWithProof,
    ) -> Self {
        Self {
            ledger_info_with_sigs,
            txn_list_with_proof,
        }
    }
}

impl fmt::Display for GetChunkResponse {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let txns_repr = match self.txn_list_with_proof.first_transaction_version {
            None => "empty".to_string(),
            Some(first_ver) => format!(
                "versions [{} - {}]",
                first_ver,
                first_ver - 1 + self.txn_list_with_proof.len() as u64
            ),
        };
        write!(
            f,
            "[ChunkResponse: ledger_info: {}, txns: {}]",
            self.ledger_info_with_sigs.ledger_info(),
            txns_repr,
        )
    }
}

impl TryFrom<network::proto::GetChunkResponse> for GetChunkResponse {
    type Error = Error;

    fn try_from(proto: network::proto::GetChunkResponse) -> Result<Self> {
        Ok(lcs::from_bytes(&proto.bytes)?)
    }
}

impl TryFrom<GetChunkResponse> for network::proto::GetChunkResponse {
    type Error = Error;

    fn try_from(chunk_response: GetChunkResponse) -> Result<Self> {
        Ok(Self {
            bytes: lcs::to_bytes(&chunk_response)?,
        })
    }
}
