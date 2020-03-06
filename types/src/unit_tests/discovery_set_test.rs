// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::discovery_set::DiscoverySet;
use lcs::test_helpers::assert_canonical_encode_decode;
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn test_discovery_set_canonical_serialization(set in any::<DiscoverySet>()) {
        assert_canonical_encode_decode(set);
    }
}
