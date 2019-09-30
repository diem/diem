// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::contract_event::{ContractEvent, EventWithProof};
use libra_proto_conv::test_helper::assert_protobuf_encode_decode;
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_event(event in any::<ContractEvent>()) {
        assert_protobuf_encode_decode(&event);
    }

    #[test]
    fn test_event_with_proof(event_with_proof in any::<EventWithProof>()) {
        assert_protobuf_encode_decode(&event_with_proof);
    }
}
