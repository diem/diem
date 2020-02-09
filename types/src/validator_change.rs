// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use crate::crypto_proxies::{EpochInfo, LedgerInfoWithSignatures};
use crate::waypoint::Waypoint;
use anyhow::{ensure, format_err, Error, Result};
#[cfg(any(test, feature = "fuzzing"))]
use proptest::{collection::vec, prelude::*};
use serde::{Deserialize, Serialize};
use std::convert::{TryFrom, TryInto};
use std::sync::Arc;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
/// A vector of LedgerInfo with contiguous increasing epoch numbers to prove a sequence of
/// validator changes from the first LedgerInfo's epoch.
pub struct ValidatorChangeProof {
    pub ledger_info_with_sigs: Vec<LedgerInfoWithSignatures>,
    pub more: bool,
}

#[derive(Clone)]
/// The verification of the validator change proof starts with some verifier that is trusted by the
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
                    "LedgerInfo has unexpected epoch"
                );
                ledger_info.verify(epoch_info.verifier.as_ref())?;
                Ok(())
            }
        }
    }

    /// Returns true in case the given epoch is larger than the existing verifier can support.
    /// In this case the ValidatorChangeProof should be verified and the verifier updated.
    pub fn epoch_change_verification_required(&self, epoch: u64) -> bool {
        match self {
            VerifierType::Waypoint(_) => true,
            VerifierType::TrustedVerifier(epoch_info) => epoch_info.epoch < epoch,
        }
    }
}

impl ValidatorChangeProof {
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
            .ok_or_else(|| format_err!("Empty ValidatorChangeProof"))
    }

    /// Verify the proof is correctly chained with known epoch and validator verifier
    /// and return the LedgerInfo to start target epoch
    /// In case waypoint is present it's going to be used for verifying the very first epoch change
    /// (it's the responsibility of the caller to not pass waypoint in case it's not needed).
    pub fn verify(&self, verifier: &VerifierType) -> Result<LedgerInfoWithSignatures> {
        ensure!(
            !self.ledger_info_with_sigs.is_empty(),
            "Empty ValidatorChangeProof"
        );
        self.ledger_info_with_sigs
            .iter()
            .try_fold(verifier.clone(), |verifier, ledger_info| {
                verifier.verify(ledger_info)?;
                // While the original verification could've been via waypoints, all the next epoch
                // changes are verified using the (already trusted) validator sets.
                ledger_info
                    .ledger_info()
                    .next_validator_set()
                    .map(|v| {
                        VerifierType::TrustedVerifier(EpochInfo {
                            epoch: ledger_info.ledger_info().epoch() + 1,
                            verifier: Arc::new(v.into()),
                        })
                    })
                    .ok_or_else(|| format_err!("LedgerInfo doesn't carry ValidatorSet"))
            })?;
        Ok(self.ledger_info_with_sigs.last().unwrap().clone())
    }
}

impl TryFrom<crate::proto::types::ValidatorChangeProof> for ValidatorChangeProof {
    type Error = Error;

    fn try_from(proto: crate::proto::types::ValidatorChangeProof) -> Result<Self> {
        let ledger_info_with_sigs = proto
            .ledger_info_with_sigs
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<_>>>()?;
        let more = proto.more;

        Ok(ValidatorChangeProof {
            ledger_info_with_sigs,
            more,
        })
    }
}

impl From<ValidatorChangeProof> for crate::proto::types::ValidatorChangeProof {
    fn from(change: ValidatorChangeProof) -> Self {
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
impl Arbitrary for ValidatorChangeProof {
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
    fn verify_validator_set_change_proof() {
        use crate::crypto_proxies::random_validator_verifier;
        use crate::ledger_info::LedgerInfo;
        use libra_crypto::hash::{CryptoHash, HashValue};
        use std::collections::BTreeMap;

        let all_epoch: Vec<u64> = (1..=10).collect();
        let mut valid_ledger_info = vec![];
        let mut validator_verifier = vec![];
        // We generate end-epoch ledger info for epoch 1 to 10, each signed by the current
        // validator set and carries next validator set.
        let (mut current_signers, mut current_verifier) = random_validator_verifier(1, None, true);
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
                    42,
                    0,
                    validator_set,
                ),
                HashValue::zero(),
            );
            let signatures = current_signers
                .iter()
                .map(|s| (s.author(), s.sign_message(ledger_info.hash()).unwrap()))
                .collect();
            valid_ledger_info.push(LedgerInfoWithSignatures::new(ledger_info, signatures));
            current_signers = next_signers;
            current_verifier = next_verifier;
        }

        // Test well-formed proof will succeed
        let proof_1 = ValidatorChangeProof::new(valid_ledger_info.clone(), /* more = */ false);
        assert!(proof_1
            .verify(&VerifierType::TrustedVerifier(EpochInfo {
                epoch: all_epoch[0],
                verifier: Arc::new(validator_verifier[0].clone())
            }))
            .is_ok());

        let proof_2 =
            ValidatorChangeProof::new(valid_ledger_info[2..5].to_vec(), /* more = */ false);
        assert!(proof_2
            .verify(&VerifierType::TrustedVerifier(EpochInfo {
                epoch: all_epoch[2],
                verifier: Arc::new(validator_verifier[2].clone())
            }))
            .is_ok());

        // Test empty proof will fail verification
        let proof_3 = ValidatorChangeProof::new(vec![], /* more = */ false);
        assert!(proof_3
            .verify(&VerifierType::TrustedVerifier(EpochInfo {
                epoch: all_epoch[0],
                verifier: Arc::new(validator_verifier[0].clone())
            }))
            .is_err());

        // Test non contiguous proof will fail
        let mut list = valid_ledger_info[3..5].to_vec();
        list.extend_from_slice(&valid_ledger_info[8..9]);
        let proof_4 = ValidatorChangeProof::new(list, /* more = */ false);
        assert!(proof_4
            .verify(&VerifierType::TrustedVerifier(EpochInfo {
                epoch: all_epoch[3],
                verifier: Arc::new(validator_verifier[3].clone())
            }))
            .is_err());

        // Test non increasing proof will fail
        let mut list = valid_ledger_info.clone();
        list.reverse();
        let proof_5 = ValidatorChangeProof::new(list, /* more = */ false);
        assert!(proof_5
            .verify(&VerifierType::TrustedVerifier(EpochInfo {
                epoch: all_epoch[9],
                verifier: Arc::new(validator_verifier[9].clone())
            }))
            .is_err());

        // Test proof with invalid signatures will fail
        let proof_6 = ValidatorChangeProof::new(
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
        let waypoint_for_1_to_2 = Waypoint::new(valid_ledger_info[0].ledger_info()).unwrap();
        assert!(proof_1
            .verify(&VerifierType::Waypoint(waypoint_for_1_to_2))
            .is_ok());

        // Waypoint to wrong epoch change fails verification.
        let waypoint_for_2_to_3 = Waypoint::new(valid_ledger_info[1].ledger_info()).unwrap();
        assert!(proof_1
            .verify(&VerifierType::Waypoint(waypoint_for_2_to_3))
            .is_err());
    }
}
