// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::test_helpers::assert_canonical_encode_decode;
use crate::validator_set::ValidatorSet;
use proptest::prelude::*;
use prost_ext::test_helpers::assert_protobuf_encode_decode;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn test_validator_set_protobuf_conversion(set in any::<ValidatorSet>()) {
        assert_protobuf_encode_decode::<crate::proto::types::ValidatorSet, ValidatorSet>(&set);
    }

    #[test]
    fn test_validator_set_canonical_serialization(set in any::<ValidatorSet>()) {
        assert_canonical_encode_decode(set);
    }
}
