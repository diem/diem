// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::crypto_proxies::ValidatorSet;
use lcs::test_helpers::assert_canonical_encode_decode;
use libra_prost_ext::test_helpers::assert_protobuf_encode_decode;
use proptest::prelude::*;

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
