// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use anyhow::{Error, Result};
use libra_types::epoch_change::EpochChangeProof;
use std::convert::{TryFrom, TryInto};

impl TryFrom<crate::proto::types::EpochChangeProof> for EpochChangeProof {
    type Error = Error;

    fn try_from(proto: crate::proto::types::EpochChangeProof) -> Result<Self> {
        let ledger_info_with_sigs = proto
            .ledger_info_with_sigs
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<_>>>()?;
        let more = proto.more;

        Ok(EpochChangeProof {
            ledger_info_with_sigs,
            more,
        })
    }
}

impl From<EpochChangeProof> for crate::proto::types::EpochChangeProof {
    fn from(change: EpochChangeProof) -> Self {
        Self {
            ledger_info_with_sigs: change
                .ledger_info_with_sigs
                .into_iter()
                .map(Into::into)
                .collect(),
            more: change.more,
        }
    }
}

#[cfg(test)]
mod tests {}
