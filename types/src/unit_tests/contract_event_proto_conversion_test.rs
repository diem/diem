// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::contract_event::{ContractEvent, EventWithProof};
use libra_prost_ext::test_helpers::assert_protobuf_encode_decode;
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_event(event in any::<ContractEvent>()) {
        assert_protobuf_encode_decode::<crate::proto::types::Event, ContractEvent>(&event);
    }

    #[test]
    fn test_event_with_proof(event_with_proof in any::<EventWithProof>()) {
        assert_protobuf_encode_decode::<crate::proto::types::EventWithProof, EventWithProof>(&event_with_proof);
    }
}
