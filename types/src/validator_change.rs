// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{contract_event::EventWithProof, ledger_info::LedgerInfoWithSignatures};
use crypto::*;
use proto_conv::{FromProto, IntoProto};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatorChangeEventWithProof<Sig> {
    ledger_info_with_sigs: LedgerInfoWithSignatures<Sig>,
    event_with_proof: EventWithProof,
}

impl<Sig: Signature> IntoProto for ValidatorChangeEventWithProof<Sig> {
    type ProtoType = crate::proto::validator_change::ValidatorChangeEventWithProof;

    fn into_proto(self) -> Self::ProtoType {
        let mut out = crate::proto::validator_change::ValidatorChangeEventWithProof::new();
        out.set_ledger_info_with_sigs(self.ledger_info_with_sigs.into_proto());
        out.set_event_with_proof(self.event_with_proof.into_proto());
        out
    }
}

impl<Sig: Signature> FromProto for ValidatorChangeEventWithProof<Sig> {
    type ProtoType = crate::proto::validator_change::ValidatorChangeEventWithProof;

    fn from_proto(mut object: Self::ProtoType) -> failure::Result<Self> {
        Ok(ValidatorChangeEventWithProof {
            ledger_info_with_sigs: LedgerInfoWithSignatures::from_proto(
                object.take_ledger_info_with_sigs(),
            )?,
            event_with_proof: EventWithProof::from_proto(object.take_event_with_proof())?,
        })
    }
}

#[cfg(any(test, feature = "testing"))]
use crypto::ed25519::*;
#[cfg(any(test, feature = "testing"))]
use proptest::prelude::*;

#[cfg(any(test, feature = "testing"))]
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

#[cfg(any(test, feature = "testing"))]
impl Arbitrary for ValidatorChangeEventWithProof<Ed25519Signature> {
    type Parameters = ();
    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        arb_validator_change_event_with_proof().boxed()
    }

    type Strategy = BoxedStrategy<Self>;
}
