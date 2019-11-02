// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{contract_event::EventWithProof, ledger_info::LedgerInfoWithSignatures};
use failure::*;
use libra_crypto::*;
use std::convert::{TryFrom, TryInto};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatorChangeEventWithProof<Sig> {
    ledger_info_with_sigs: LedgerInfoWithSignatures<Sig>,
    event_with_proof: EventWithProof,
}

impl<Sig: Signature> TryFrom<crate::proto::types::ValidatorChangeEventWithProof>
    for ValidatorChangeEventWithProof<Sig>
{
    type Error = Error;

    fn try_from(proto: crate::proto::types::ValidatorChangeEventWithProof) -> Result<Self> {
        let ledger_info_with_sigs = proto
            .ledger_info_with_sigs
            .ok_or_else(|| format_err!("Missing ledger_info_with_sigs"))?
            .try_into()?;
        let event_with_proof = proto
            .event_with_proof
            .ok_or_else(|| format_err!("Missing event_with_proof"))?
            .try_into()?;
        Ok(ValidatorChangeEventWithProof {
            ledger_info_with_sigs,
            event_with_proof,
        })
    }
}

impl<Sig: Signature> From<ValidatorChangeEventWithProof<Sig>>
    for crate::proto::types::ValidatorChangeEventWithProof
{
    fn from(change: ValidatorChangeEventWithProof<Sig>) -> Self {
        Self {
            ledger_info_with_sigs: Some(change.ledger_info_with_sigs.into()),
            event_with_proof: Some(change.event_with_proof.into()),
        }
    }
}

#[cfg(any(test, feature = "fuzzing"))]
use libra_crypto::ed25519::*;
#[cfg(any(test, feature = "fuzzing"))]
use proptest::prelude::*;

#[cfg(any(test, feature = "fuzzing"))]
prop_compose! {
    fn arb_validator_change_event_with_proof()(
        ledger_info_with_sigs in any::<LedgerInfoWithSignatures<Ed25519Signature>>(),
        event_with_proof in any::<EventWithProof>(),
    ) -> ValidatorChangeEventWithProof<Ed25519Signature> {
        ValidatorChangeEventWithProof{
            ledger_info_with_sigs, event_with_proof
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
