// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::write_set::WriteSet;
use proptest::prelude::*;
use proto_conv::test_helper::assert_protobuf_encode_decode;

proptest! {
    #[test]
    fn write_set_roundtrip(write_set in any::<WriteSet>()) {
        assert_protobuf_encode_decode(&write_set);
    }
}
