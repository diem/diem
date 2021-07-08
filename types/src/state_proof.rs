// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    epoch_change::EpochChangeProof,
    ledger_info::{LedgerInfo, LedgerInfoWithSignatures},
    proof::AccumulatorConsistencyProof,
};
#[cfg(any(test, feature = "fuzzing"))]
use proptest_derive::Arbitrary;
use serde::{Deserialize, Serialize};

/// A convenience type for the collection of sub-proofs that consistitute a
/// response to a `get_state_proof` request.
///
/// From a `StateProof` response, a client should be able to ratchet their
/// [`TrustedState`] to the last epoch change LI in the [`EpochChangeProof`]
/// or the latest [`LedgerInfoWithSignatures`] if the epoch changes get them into
/// the most recent epoch.
///
/// [`TrustedState`]: crate::trusted_state::TrustedState
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
pub struct StateProof {
    latest_li_w_sigs: LedgerInfoWithSignatures,
    epoch_changes: EpochChangeProof,
    consistency_proof: AccumulatorConsistencyProof,
}

impl StateProof {
    pub fn new(
        latest_li_w_sigs: LedgerInfoWithSignatures,
        epoch_changes: EpochChangeProof,
        consistency_proof: AccumulatorConsistencyProof,
    ) -> Self {
        Self {
            latest_li_w_sigs,
            epoch_changes,
            consistency_proof,
        }
    }

    pub fn into_inner(
        self,
    ) -> (
        LedgerInfoWithSignatures,
        EpochChangeProof,
        AccumulatorConsistencyProof,
    ) {
        (
            self.latest_li_w_sigs,
            self.epoch_changes,
            self.consistency_proof,
        )
    }

    pub fn as_inner(
        &self,
    ) -> (
        &LedgerInfoWithSignatures,
        &EpochChangeProof,
        &AccumulatorConsistencyProof,
    ) {
        (
            &self.latest_li_w_sigs,
            &self.epoch_changes,
            &self.consistency_proof,
        )
    }

    #[inline]
    pub fn latest_ledger_info(&self) -> &LedgerInfo {
        self.latest_li_w_sigs.ledger_info()
    }

    #[inline]
    pub fn latest_ledger_info_w_sigs(&self) -> &LedgerInfoWithSignatures {
        &self.latest_li_w_sigs
    }

    #[inline]
    pub fn epoch_changes(&self) -> &EpochChangeProof {
        &self.epoch_changes
    }

    #[inline]
    pub fn consistency_proof(&self) -> &AccumulatorConsistencyProof {
        &self.consistency_proof
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bcs::test_helpers::assert_canonical_encode_decode;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(20))]

        #[test]
        fn test_state_proof_canonical_serialization(proof in any::<StateProof>()) {
            assert_canonical_encode_decode(proof);
        }
    }
}
