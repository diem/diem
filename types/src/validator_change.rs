// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![allow(clippy::unit_arg)]

use crate::{contract_event::EventWithProof, ledger_info::LedgerInfoWithSignatures};
use proptest_derive::Arbitrary;
use proto_conv::{FromProto, IntoProto};

#[derive(Arbitrary, Clone, Debug, Eq, PartialEq, FromProto, IntoProto)]
#[ProtoType(crate::proto::validator_change::ValidatorChangeEventWithProof)]
pub struct ValidatorChangeEventWithProof {
    ledger_info_with_sigs: LedgerInfoWithSignatures,
    event_with_proof: EventWithProof,
}
