// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::contract_event::{ContractEvent, EventWithProof};
use lcs::test_helpers::assert_canonical_encode_decode;
use proptest::prelude::*;

proptest! {
    #[test]
    fn event_lcs_roundtrip(event in any::<ContractEvent>()) {
        assert_canonical_encode_decode(event);
    }

    #[test]
    fn event_with_proof_lcs_roundtrip(event_with_proof in any::<EventWithProof>()) {
        assert_canonical_encode_decode(event_with_proof);
    }
}
