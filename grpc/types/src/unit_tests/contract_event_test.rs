// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_prost_test_helpers::assert_protobuf_encode_decode;
use libra_types::contract_event::{ContractEvent, EventWithProof};
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
}
