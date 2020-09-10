// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::on_chain_config::ValidatorSet;
use lcs::test_helpers::assert_canonical_encode_decode;
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn test_validator_set_canonical_serialization(set in any::<ValidatorSet>()) {
        assert_canonical_encode_decode(set);
    }
}
