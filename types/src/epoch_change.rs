// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use crate::{
    epoch_info::EpochInfo,
    ledger_info::{LedgerInfo, LedgerInfoWithSignatures},
    waypoint::Waypoint,
};
use anyhow::{ensure, format_err, Error, Result};
#[cfg(any(test, feature = "fuzzing"))]
use proptest::{collection::vec, prelude::*};
use serde::{Deserialize, Serialize};
use std::{
    convert::{TryFrom, TryInto},
    sync::Arc,
};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
/// A vector of LedgerInfo with contiguous increasing epoch numbers to prove a sequence of
/// epoch changes from the first LedgerInfo's epoch.
pub struct EpochChangeProof {
    pub ledger_info_with_sigs: Vec<LedgerInfoWithSignatures>,
    pub more: bool,
}

#[derive(Clone, Debug)]
/// The verification of the epoch change proof starts with some verifier that is trusted by the
/// client: could be either a waypoint (upon startup) or a known validator verifier.
pub enum VerifierType {
    Waypoint(Waypoint),
    TrustedVerifier(EpochInfo),
}

impl VerifierType {
    pub fn verify(&self, ledger_info: &LedgerInfoWithSignatures) -> Result<()> {
        match self {
            VerifierType::Waypoint(w) => w.verify(ledger_info.ledger_info()),
            VerifierType::TrustedVerifier(epoch_info) => {
                ensure!(
                    epoch_info.epoch == ledger_info.ledger_info().epoch(),
                    "LedgerInfo has unexpected epoch {}, expected {}",
                    ledger_info.ledger_info().epoch(),
                    epoch_info.epoch
                );
                ledger_info.verify_signatures(epoch_info.verifier.as_ref())?;
                Ok(())
            }
        }
    }

    /// Returns true in case the given epoch is larger than the existing verifier can support.
    /// In this case the EpochChangeProof should be verified and the verifier updated.
    pub fn epoch_change_verification_required(&self, epoch: u64) -> bool {
        match self {
            VerifierType::Waypoint(_) => true,
            VerifierType::TrustedVerifier(epoch_info) => epoch_info.epoch < epoch,
        }
    }

    /// Returns true if the given [`LedgerInfo`] is stale and probably in our
    /// trusted prefix.
    ///
    /// For example, if we have a waypoint with version 5, an epoch change ledger
    /// info with version 3 < 5 is already in our trusted prefix and so we can
    /// ignore it.
    ///
    /// Likewise, if we're in epoch 10 with the corresponding validator set, an
    /// epoch change ledger info with epoch 6 can be safely ignored.
    pub fn is_ledger_info_stale(&self, ledger_info: &LedgerInfo) -> bool {
        match self {
            VerifierType::Waypoint(waypoint) => ledger_info.version() < waypoint.version(),
            VerifierType::TrustedVerifier(epoch_info) => ledger_info.epoch() < epoch_info.epoch,
        }
    }
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
    pub fn verify(&self, verifier: &VerifierType) -> Result<&LedgerInfoWithSignatures> {
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

        self.ledger_info_with_sigs
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
            // Try to verify each (epoch -> epoch + 1) jump in the EpochChangeProof.
            .try_fold(verifier.clone(), |verifier, ledger_info_with_sigs| {
                verifier.verify(ledger_info_with_sigs)?;
                // While the original verification could've been via waypoints,
                // all the next epoch changes are verified using the (already
                // trusted) validator sets.
                ledger_info_with_sigs
                    .ledger_info()
                    .next_validator_set()
                    .map(|v| {
                        VerifierType::TrustedVerifier(EpochInfo {
                            epoch: ledger_info_with_sigs.ledger_info().epoch() + 1,
                            verifier: Arc::new(v.into()),
                        })
                    })
                    .ok_or_else(|| format_err!("LedgerInfo doesn't carry a ValidatorSet"))
            })?;

        Ok(self.ledger_info_with_sigs.last().unwrap())
    }
}

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
    use crate::block_info::BlockInfo;

    #[test]
    fn verify_epoch_change_proof() {
        use crate::{ledger_info::LedgerInfo, validator_verifier::random_validator_verifier};
        use libra_crypto::hash::{CryptoHash, HashValue};
        use std::collections::BTreeMap;

        let all_epoch: Vec<u64> = (1..=10).collect();
        let mut valid_ledger_info = vec![];
        let mut validator_verifier = vec![];

        // We generate end-epoch ledger info for epoch 1 to 10, each signed by the current
        // validator set and carrying the next validator set.
        let (mut current_signers, mut current_verifier) = random_validator_verifier(1, None, true);
        let mut current_version = 123;
        for epoch in &all_epoch {
            validator_verifier.push(current_verifier.clone());
            let (next_signers, next_verifier) =
                random_validator_verifier((*epoch + 1) as usize, None, true);
            let validator_set = Some((&next_verifier).into());
            let ledger_info = LedgerInfo::new(
                BlockInfo::new(
                    *epoch,
                    0,
                    HashValue::zero(),
                    HashValue::zero(),
                    current_version,
                    0,
                    validator_set,
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
            .verify(&VerifierType::TrustedVerifier(EpochInfo {
                epoch: all_epoch[0],
                verifier: Arc::new(validator_verifier[0].clone())
            }))
            .is_ok());

        let proof_2 =
            EpochChangeProof::new(valid_ledger_info[2..5].to_vec(), /* more = */ false);
        assert!(proof_2
            .verify(&VerifierType::TrustedVerifier(EpochInfo {
                epoch: all_epoch[2],
                verifier: Arc::new(validator_verifier[2].clone())
            }))
            .is_ok());

        // Test proof with stale prefix will verify
        assert!(proof_1
            .verify(&VerifierType::TrustedVerifier(EpochInfo {
                epoch: all_epoch[4],
                verifier: Arc::new(validator_verifier[4].clone())
            }))
            .is_ok());

        // Test empty proof will fail verification
        let proof_3 = EpochChangeProof::new(vec![], /* more = */ false);
        assert!(proof_3
            .verify(&VerifierType::TrustedVerifier(EpochInfo {
                epoch: all_epoch[0],
                verifier: Arc::new(validator_verifier[0].clone())
            }))
            .is_err());

        // Test non contiguous proof will fail
        let mut list = valid_ledger_info[3..5].to_vec();
        list.extend_from_slice(&valid_ledger_info[8..9]);
        let proof_4 = EpochChangeProof::new(list, /* more = */ false);
        assert!(proof_4
            .verify(&VerifierType::TrustedVerifier(EpochInfo {
                epoch: all_epoch[3],
                verifier: Arc::new(validator_verifier[3].clone())
            }))
            .is_err());

        // Test non increasing proof will fail
        let mut list = valid_ledger_info.clone();
        list.reverse();
        let proof_5 = EpochChangeProof::new(list, /* more = */ false);
        assert!(proof_5
            .verify(&VerifierType::TrustedVerifier(EpochInfo {
                epoch: all_epoch[9],
                verifier: Arc::new(validator_verifier[9].clone())
            }))
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
            .verify(&VerifierType::TrustedVerifier(EpochInfo {
                epoch: all_epoch[0],
                verifier: Arc::new(validator_verifier[0].clone())
            }))
            .is_err());

        // Test proof with waypoint corresponding to the first epoch change succeeds.
        let waypoint_for_1_to_2 =
            Waypoint::new_epoch_boundary(valid_ledger_info[0].ledger_info()).unwrap();
        assert!(proof_1
            .verify(&VerifierType::Waypoint(waypoint_for_1_to_2))
            .is_ok());

        // Test proof with stale prefix will verify with a Waypoint
        let waypoint_for_5_to_6 =
            Waypoint::new_epoch_boundary(valid_ledger_info[4].ledger_info()).unwrap();
        assert!(proof_1
            .verify(&VerifierType::Waypoint(waypoint_for_5_to_6))
            .is_ok());

        // Waypoint before proof range will fail to verify
        let waypoint_for_3_to_4 =
            Waypoint::new_epoch_boundary(valid_ledger_info[2].ledger_info()).unwrap();
        let proof_7 =
            EpochChangeProof::new(valid_ledger_info[4..8].to_vec(), /* more */ false);
        assert!(proof_7
            .verify(&VerifierType::Waypoint(waypoint_for_3_to_4))
            .is_err());

        // Waypoint after proof range will fail to verify
        let proof_8 = EpochChangeProof::new(valid_ledger_info[..1].to_vec(), /* more */ false);
        assert!(proof_8
            .verify(&VerifierType::Waypoint(waypoint_for_3_to_4))
            .is_err());
    }
}
