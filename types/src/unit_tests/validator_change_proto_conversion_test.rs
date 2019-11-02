// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::validator_change::ValidatorChangeEventWithProof;
use libra_crypto::ed25519::*;
use libra_prost_ext::test_helpers::assert_protobuf_encode_decode;
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn test_validator_change_event_with_proof_conversion(
        change in any::<ValidatorChangeEventWithProof<Ed25519Signature>>()
    ) {
        assert_protobuf_encode_decode::<crate::proto::types::ValidatorChangeEventWithProof, ValidatorChangeEventWithProof<_>>(&change);
    }
}
