// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use libra_schemadb::schema::assert_encode_decode;
use proptest::prelude::*;

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
