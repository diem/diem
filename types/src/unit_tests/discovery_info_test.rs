// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::discovery_info::FullNodeDiscoveryInfo;
use lcs::test_helpers::assert_canonical_encode_decode;
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_discovery_info_canonical_serialization(info in any::<FullNodeDiscoveryInfo>()) {
        assert_canonical_encode_decode(info);
    }
}
