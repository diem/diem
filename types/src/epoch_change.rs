// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use crate::ledger_info::{LedgerInfo, LedgerInfoWithSignatures};
use anyhow::{ensure, format_err, Result};
#[cfg(any(test, feature = "fuzzing"))]
use proptest::{collection::vec, prelude::*};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// The verification of the epoch change proof starts with verifier that is trusted by the
/// client: could be either a waypoint (upon startup) or a known epoch info.
pub trait Verifier: Debug + Send + Sync {
    /// Verify if the ledger_info is trust worthy.
    fn verify(&self, ledger_info: &LedgerInfoWithSignatures) -> Result<()>;

    /// Returns true in case the given epoch is larger than the existing verifier can support.
    /// In this case the EpochChangeProof should be verified and the verifier updated.
    fn epoch_change_verification_required(&self, epoch: u64) -> bool;

    /// Returns true if the given [`LedgerInfo`] is stale and probably in our
    /// trusted prefix.
    ///
    /// For example, if we have a waypoint with version 5, an epoch change ledger
    /// info with version 3 < 5 is already in our trusted prefix and so we can
    /// ignore it.
    ///
    /// Likewise, if we're in epoch 10 with the corresponding validator set, an
    /// epoch change ledger info with epoch 6 can be safely ignored.
    fn is_ledger_info_stale(&self, ledger_info: &LedgerInfo) -> bool;
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
/// A vector of LedgerInfo with contiguous increasing epoch numbers to prove a sequence of
/// epoch changes from the first LedgerInfo's epoch.
pub struct EpochChangeProof {
    pub ledger_info_with_sigs: Vec<LedgerInfoWithSignatures>,
    pub more: bool,
}
impl EpochChangeProof {
    pub fn new(ledger_info_with_sigs: Vec<LedgerInfoWithSignatures>, more: bool) -> Self {
        Self {
            ledger_info_with_sigs,
            more,
        }
    }

    /// The first/lowest epoch of the proof to indicate which epoch this proof is helping with
    pub fn epoch(&self) -> Result<u64> {
        self.ledger_info_with_sigs
            .first()
            .map(|li| li.ledger_info().epoch())
            .ok_or_else(|| format_err!("Empty EpochChangeProof"))
    }

    /// Verify the proof is correctly chained with known epoch and validator
    /// verifier and return the [`LedgerInfoWithSignatures`] to start target epoch.
    ///
    /// In case a waypoint is present, it's going to be used for verifying the
    /// very first epoch change (it's the responsibility of the caller to not
    /// pass a waypoint in case it's not needed).
    ///
    /// We will also skip any stale ledger info's in the [`EpochChangeProof`].
    pub fn verify(&self, verifier: &dyn Verifier) -> Result<&LedgerInfoWithSignatures> {
        ensure!(
            !self.ledger_info_with_sigs.is_empty(),
            "The EpochChangeProof is empty"
        );
        ensure!(
            !verifier
                .is_ledger_info_stale(self.ledger_info_with_sigs.last().unwrap().ledger_info()),
            "The EpochChangeProof is stale as our verifier is already ahead \
             of the entire EpochChangeProof"
        );
        let mut verifier_ref = verifier;

        for ledger_info_with_sigs in self
            .ledger_info_with_sigs
            .iter()
            // Skip any stale ledger infos in the proof prefix. Note that with
            // the assertion above, we are guaranteed there is at least one
            // non-stale ledger info in the proof.
            //
            // It's useful to skip these stale ledger infos to better allow for
            // concurrent client requests.
            //
            // For example, suppose the following:
            //
            // 1. My current trusted state is at epoch 5.
            // 2. I make two concurrent requests to two validators A and B, who
            //    live at epochs 9 and 11 respectively.
            //
            // If A's response returns first, I will ratchet my trusted state
            // to epoch 9. When B's response returns, I will still be able to
            // ratchet forward to 11 even though B's EpochChangeProof
            // includes a bunch of stale ledger infos (for epochs 5, 6, 7, 8).
            //
            // Of course, if B's response returns first, we will reject A's
            // response as it's completely stale.
            .skip_while(|&ledger_info_with_sigs| {
                verifier.is_ledger_info_stale(ledger_info_with_sigs.ledger_info())
            })
        {
            // Try to verify each (epoch -> epoch + 1) jump in the EpochChangeProof.
            verifier_ref.verify(ledger_info_with_sigs)?;
            // While the original verification could've been via waypoints,
            // all the next epoch changes are verified using the (already
            // trusted) validator sets.
            verifier_ref = ledger_info_with_sigs
                .ledger_info()
                .next_epoch_info()
                .ok_or_else(|| format_err!("LedgerInfo doesn't carry a ValidatorSet"))?;
        }

        Ok(self.ledger_info_with_sigs.last().unwrap())
    }
}

#[cfg(any(test, feature = "fuzzing"))]
impl Arbitrary for EpochChangeProof {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        (vec(any::<LedgerInfoWithSignatures>(), 0..10), any::<bool>())
            .prop_map(|(ledger_infos_with_sigs, more)| Self::new(ledger_infos_with_sigs, more))
            .boxed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{block_info::BlockInfo, epoch_info::EpochInfo, waypoint::Waypoint};

    #[test]
    fn verify_epoch_change_proof() {
        use crate::{ledger_info::LedgerInfo, validator_verifier::random_validator_verifier};
        use libra_crypto::hash::{CryptoHash, HashValue};
        use std::collections::BTreeMap;

        let all_epoch: Vec<u64> = (1..=10).collect();
        let mut valid_ledger_info = vec![];
        let mut validator_verifier = vec![];

        // We generate end-epoch ledger info for epoch 1 to 10, each signed by the current
        // validator set and carrying the next epoch info.
        let (mut current_signers, mut current_verifier) = random_validator_verifier(1, None, true);
        let mut current_version = 123;
        for epoch in &all_epoch {
            validator_verifier.push(current_verifier.clone());
            let (next_signers, next_verifier) =
                random_validator_verifier((*epoch + 1) as usize, None, true);
            let epoch_info = EpochInfo {
                epoch: *epoch + 1,
                verifier: next_verifier.clone(),
            };
            let ledger_info = LedgerInfo::new(
                BlockInfo::new(
                    *epoch,
                    0,
                    HashValue::zero(),
                    HashValue::zero(),
                    current_version,
                    0,
                    Some(epoch_info),
                ),
                HashValue::zero(),
            );
            let signatures = current_signers
                .iter()
                .map(|s| (s.author(), s.sign_message(ledger_info.hash())))
                .collect();
            valid_ledger_info.push(LedgerInfoWithSignatures::new(ledger_info, signatures));
            current_signers = next_signers;
            current_verifier = next_verifier;
            current_version += 1;
        }

        // Test well-formed proof will succeed
        let proof_1 = EpochChangeProof::new(valid_ledger_info.clone(), /* more = */ false);
        assert!(proof_1
            .verify(&EpochInfo {
                epoch: all_epoch[0],
                verifier: validator_verifier[0].clone(),
            })
            .is_ok());

        let proof_2 =
            EpochChangeProof::new(valid_ledger_info[2..5].to_vec(), /* more = */ false);
        assert!(proof_2
            .verify(&EpochInfo {
                epoch: all_epoch[2],
                verifier: validator_verifier[2].clone()
            })
            .is_ok());

        // Test proof with stale prefix will verify
        assert!(proof_1
            .verify(&EpochInfo {
                epoch: all_epoch[4],
                verifier: validator_verifier[4].clone()
            })
            .is_ok());

        // Test empty proof will fail verification
        let proof_3 = EpochChangeProof::new(vec![], /* more = */ false);
        assert!(proof_3
            .verify(&EpochInfo {
                epoch: all_epoch[0],
                verifier: validator_verifier[0].clone()
            })
            .is_err());

        // Test non contiguous proof will fail
        let mut list = valid_ledger_info[3..5].to_vec();
        list.extend_from_slice(&valid_ledger_info[8..9]);
        let proof_4 = EpochChangeProof::new(list, /* more = */ false);
        assert!(proof_4
            .verify(&EpochInfo {
                epoch: all_epoch[3],
                verifier: validator_verifier[3].clone()
            })
            .is_err());

        // Test non increasing proof will fail
        let mut list = valid_ledger_info.clone();
        list.reverse();
        let proof_5 = EpochChangeProof::new(list, /* more = */ false);
        assert!(proof_5
            .verify(&EpochInfo {
                epoch: all_epoch[9],
                verifier: validator_verifier[9].clone()
            })
            .is_err());

        // Test proof with invalid signatures will fail
        let proof_6 = EpochChangeProof::new(
            vec![LedgerInfoWithSignatures::new(
                valid_ledger_info[0].ledger_info().clone(),
                BTreeMap::new(),
            )],
            /* more = */ false,
        );
        assert!(proof_6
            .verify(&EpochInfo {
                epoch: all_epoch[0],
                verifier: validator_verifier[0].clone()
            })
            .is_err());

        // Test proof with waypoint corresponding to the first epoch change succeeds.
        let waypoint_for_1_to_2 =
            Waypoint::new_epoch_boundary(valid_ledger_info[0].ledger_info()).unwrap();
        assert!(proof_1.verify(&waypoint_for_1_to_2).is_ok());

        // Test proof with stale prefix will verify with a Waypoint
        let waypoint_for_5_to_6 =
            Waypoint::new_epoch_boundary(valid_ledger_info[4].ledger_info()).unwrap();
        assert!(proof_1.verify(&waypoint_for_5_to_6).is_ok());

        // Waypoint before proof range will fail to verify
        let waypoint_for_3_to_4 =
            Waypoint::new_epoch_boundary(valid_ledger_info[2].ledger_info()).unwrap();
        let proof_7 =
            EpochChangeProof::new(valid_ledger_info[4..8].to_vec(), /* more */ false);
        assert!(proof_7.verify(&waypoint_for_3_to_4).is_err());

        // Waypoint after proof range will fail to verify
        let proof_8 = EpochChangeProof::new(valid_ledger_info[..1].to_vec(), /* more */ false);
        assert!(proof_8.verify(&waypoint_for_3_to_4).is_err());
    }
}
