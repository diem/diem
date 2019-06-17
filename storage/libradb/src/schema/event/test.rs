// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use proptest::prelude::*;
use schemadb::schema::assert_encode_decode;

proptest! {
    #[test]
    fn test_encode_decode(
        version in any::<Version>(),
        index in any::<u64>(),
        event in any::<ContractEvent>(),
    ) {
        assert_encode_decode::<EventSchema>(&(version, index), &event);
    }
}
