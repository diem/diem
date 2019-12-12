// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{proptest_types::arb_multiaddr, test_helpers::assert_canonical_encode_decode};
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_multiaddr_canonical_serialization(multiaddr in arb_multiaddr()) {
        assert_canonical_encode_decode(multiaddr);
    }
}
