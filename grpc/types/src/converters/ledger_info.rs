// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{Error, Result};
use libra_crypto::hash::HashValue;
use libra_types::{
    block_info::BlockInfo,
    ledger_info::{LedgerInfo, LedgerInfoWithSignatures},
};
use std::convert::TryFrom;

impl TryFrom<crate::proto::types::LedgerInfo> for LedgerInfo {
    type Error = Error;

    fn try_from(proto: crate::proto::types::LedgerInfo) -> Result<Self> {
        let version = proto.version;
        let transaction_accumulator_hash =
            HashValue::from_slice(&proto.transaction_accumulator_hash)?;
        let consensus_data_hash = HashValue::from_slice(&proto.consensus_data_hash)?;
        let consensus_block_id = HashValue::from_slice(&proto.consensus_block_id)?;
        let epoch = proto.epoch;
        let round = proto.round;
        let timestamp_usecs = proto.timestamp_usecs;

        let next_epoch_info = lcs::from_bytes(&proto.next_epoch_info)?;
        Ok(LedgerInfo::new(
            BlockInfo::new(
                epoch,
                round,
                consensus_block_id,
                transaction_accumulator_hash,
                version,
                timestamp_usecs,
                next_epoch_info,
            ),
            consensus_data_hash,
        ))
    }
}

impl From<LedgerInfo> for crate::proto::types::LedgerInfo {
    fn from(ledger_info: LedgerInfo) -> Self {
        Self {
            version: ledger_info.version(),
            transaction_accumulator_hash: ledger_info.transaction_accumulator_hash().to_vec(),
            consensus_data_hash: ledger_info.consensus_data_hash().to_vec(),
            consensus_block_id: ledger_info.consensus_block_id().to_vec(),
            epoch: ledger_info.epoch(),
            round: ledger_info.round(),
            timestamp_usecs: ledger_info.timestamp_usecs(),
            next_epoch_info: lcs::to_bytes(&ledger_info.next_epoch_info())
                .expect("failed to serialize EpochInfo"),
        }
    }
}

impl TryFrom<crate::proto::types::LedgerInfoWithSignatures> for LedgerInfoWithSignatures {
    type Error = Error;

    fn try_from(proto: crate::proto::types::LedgerInfoWithSignatures) -> Result<Self> {
        Ok(lcs::from_bytes(&proto.bytes)?)
    }
}

impl From<LedgerInfoWithSignatures> for crate::proto::types::LedgerInfoWithSignatures {
    fn from(ledger_info_with_sigs: LedgerInfoWithSignatures) -> Self {
        Self {
            bytes: lcs::to_bytes(&ledger_info_with_sigs).expect("failed to serialize ledger info"),
        }
    }
}

#[cfg(test)]
mod tests {}
