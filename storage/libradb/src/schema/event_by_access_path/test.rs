// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use proptest::prelude::*;
use schemadb::schema::assert_encode_decode;

proptest! {
    #[test]
    fn test_encode_decode(
        access_path in any::<AccessPath>(),
        seq_num in any::<u64>(),
        version in any::<Version>(),
        index in any::<u64>(),
    ) {
        assert_encode_decode::<EventByAccessPathSchema>(&(access_path, seq_num), &(version, index));
    }
}
