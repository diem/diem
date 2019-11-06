// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use proptest::prelude::*;
use schemadb::schema::assert_encode_decode;

proptest! {
    #[test]
    fn test_encode_decode(
        version in any::<Version>(),
        epoch_num in any::<u64>(),
    ) {
        assert_encode_decode::<EpochByVersionSchema>(&version, &epoch_num);
    }
}
