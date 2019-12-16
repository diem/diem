// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::validator_change::ValidatorChangeProof;
use libra_prost_ext::test_helpers::assert_protobuf_encode_decode;
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn test_validator_change_proof_conversion(change in any::<ValidatorChangeProof>()) {
        assert_protobuf_encode_decode::<crate::proto::types::ValidatorChangeProof, ValidatorChangeProof>(&change);
    }
}
