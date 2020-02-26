// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{test_helpers::assert_canonical_encode_decode, write_set::WriteSet};
use proptest::prelude::*;

proptest! {
    #[test]
    fn write_set_roundtrip_canonical_serialization(write_set in any::<WriteSet>()) {
        assert_canonical_encode_decode(write_set);
    }
}
