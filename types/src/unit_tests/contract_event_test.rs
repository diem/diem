// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    contract_event::{ContractEvent, EventWithProof},
    test_helpers::assert_canonical_encode_decode,
};
use libra_prost_ext::test_helpers::assert_protobuf_encode_decode;
use proptest::prelude::*;

proptest! {
    #[test]
    fn event_proto_roundtrip(event in any::<ContractEvent>()) {
        assert_protobuf_encode_decode::<crate::proto::types::Event, ContractEvent>(&event);
    }

    #[test]
    fn event_with_proof_proto_roundtrip(event_with_proof in any::<EventWithProof>()) {
        assert_protobuf_encode_decode::<crate::proto::types::EventWithProof, EventWithProof>(&event_with_proof);
    }

    #[test]
    fn event_lcs_roundtrip(event in any::<ContractEvent>()) {
        assert_canonical_encode_decode(event);
    }

    #[test]
    fn event_with_proof_lcs_roundtrip(event_with_proof in any::<EventWithProof>()) {
        assert_canonical_encode_decode(event_with_proof);
    }
}
