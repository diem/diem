// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use crate::crypto_proxies::ValidatorVerifier;
use crate::ledger_info::LedgerInfoWithSignatures;
use failure::*;
use libra_crypto::ed25519::*;
use libra_crypto::*;
use std::convert::{TryFrom, TryInto};

#[derive(Clone, Debug, Eq, PartialEq)]
/// A vector of LedgerInfo with contiguous increasing epoch numbers to prove a sequence of
/// validator changes from the first LedgerInfo's epoch.
pub struct ValidatorChangeEventWithProof<Sig> {
    pub ledger_info_with_sigs: Vec<LedgerInfoWithSignatures<Sig>>,
}

impl<Sig: Signature> ValidatorChangeEventWithProof<Sig> {
    pub fn new(ledger_info_with_sigs: Vec<LedgerInfoWithSignatures<Sig>>) -> Self {
        Self {
            ledger_info_with_sigs,
        }
    }

    /// The first/lowest epoch of the proof to indicate which epoch this proof is helping with
    pub fn epoch(&self) -> Result<u64> {
        self.ledger_info_with_sigs
            .first()
            .map(|li| li.ledger_info().epoch())
            .ok_or_else(|| format_err!("Empty ValidatorChangeEventWithProof"))
    }
}
impl ValidatorChangeEventWithProof<Ed25519Signature> {
    /// Verify the proof is correctly chained with known epoch and validator verifier
    /// and return the LedgerInfo to start target epoch
    pub fn verify(
        &self,
        epoch: u64,
        validator_verifier: &ValidatorVerifier,
    ) -> Result<LedgerInfoWithSignatures<Ed25519Signature>> {
        ensure!(
            !self.ledger_info_with_sigs.is_empty(),
            "Empty ValidatorChangeEventWithProof"
        );
        self.ledger_info_with_sigs.iter().try_fold(
            (epoch, validator_verifier.clone()),
            |(epoch, validator_verifier), ledger_info| {
                ensure!(
                    epoch == ledger_info.ledger_info().epoch(),
                    "LedgerInfo has unexpected epoch"
                );
                ledger_info.verify(&validator_verifier)?;
                ledger_info
                    .ledger_info()
                    .next_validator_set()
                    .map(|v| (epoch + 1, v.into()))
                    .ok_or_else(|| format_err!("LedgerInfo doesn't carry ValidatorSet"))
            },
        )?;
        Ok(self.ledger_info_with_sigs.last().unwrap().clone())
    }
}

#[test]
fn verify_validator_set_change_proof() {
    use crate::crypto_proxies::random_validator_verifier;
    use crate::ledger_info::LedgerInfo;
    use libra_crypto::hash::CryptoHash;
    use std::collections::BTreeMap;

    let all_epoch: Vec<u64> = (1..=10).collect();
    let mut valid_ledger_info = vec![];
    let mut validator_verifier = vec![];
    // We generate end-epoch ledger info for epoch 1 to 10, each signed by the current validator set and carries next validator set.
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
    let proof_1 = ValidatorChangeEventWithProof::new(valid_ledger_info.clone());
    assert!(proof_1.verify(all_epoch[0], &validator_verifier[0]).is_ok());

    let proof_2 = ValidatorChangeEventWithProof::new(valid_ledger_info[2..5].to_vec());
    assert!(proof_2.verify(all_epoch[2], &validator_verifier[2]).is_ok());

    // Test empty proof will fail verification
    let proof_3 = ValidatorChangeEventWithProof::new(vec![]);
    assert!(proof_3
        .verify(all_epoch[0], &validator_verifier[0])
        .is_err());

    // Test non contiguous proof will fail
    let mut list = valid_ledger_info[3..5].to_vec();
    list.extend_from_slice(&valid_ledger_info[8..9]);
    let proof_4 = ValidatorChangeEventWithProof::new(list);
    assert!(proof_4
        .verify(all_epoch[3], &validator_verifier[3])
        .is_err());

    // Test non increasing proof will fail
    let mut list = valid_ledger_info.clone();
    list.reverse();
    let proof_5 = ValidatorChangeEventWithProof::new(list);
    assert!(proof_5
        .verify(all_epoch[9], &validator_verifier[9])
        .is_err());

    // Test proof with invalid signatures will fail
    let proof_6 = ValidatorChangeEventWithProof::new(vec![LedgerInfoWithSignatures::new(
        valid_ledger_info[0].ledger_info().clone(),
        BTreeMap::new(),
    )]);
    assert!(proof_6
        .verify(all_epoch[0], &validator_verifier[0])
        .is_err());
}

impl<Sig: Signature> TryFrom<crate::proto::types::ValidatorChangeEventWithProof>
    for ValidatorChangeEventWithProof<Sig>
{
    type Error = Error;

    fn try_from(proto: crate::proto::types::ValidatorChangeEventWithProof) -> Result<Self> {
        let ledger_info_with_sigs = proto
            .ledger_info_with_sigs
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<_>>>()?;
        Ok(ValidatorChangeEventWithProof {
            ledger_info_with_sigs,
        })
    }
}

impl<Sig: Signature> From<ValidatorChangeEventWithProof<Sig>>
    for crate::proto::types::ValidatorChangeEventWithProof
{
    fn from(change: ValidatorChangeEventWithProof<Sig>) -> Self {
        Self {
            ledger_info_with_sigs: change
                .ledger_info_with_sigs
                .into_iter()
                .map(Into::into)
                .collect(),
        }
    }
}

#[allow(unused_imports)]
use crate::block_info::BlockInfo;
#[cfg(any(test, feature = "fuzzing"))]
use proptest::collection::vec;
#[cfg(any(test, feature = "fuzzing"))]
use proptest::prelude::*;
#[cfg(any(test, feature = "fuzzing"))]
prop_compose! {
    fn arb_validator_change_event_with_proof()(
        ledger_info_with_sigs in vec(any::<LedgerInfoWithSignatures<Ed25519Signature>>(), 0..10),
    ) -> ValidatorChangeEventWithProof<Ed25519Signature> {
        ValidatorChangeEventWithProof{
            ledger_info_with_sigs
        }
    }
}

#[cfg(any(test, feature = "fuzzing"))]
impl Arbitrary for ValidatorChangeEventWithProof<Ed25519Signature> {
    type Parameters = ();
    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        arb_validator_change_event_with_proof().boxed()
    }

    type Strategy = BoxedStrategy<Self>;
}
