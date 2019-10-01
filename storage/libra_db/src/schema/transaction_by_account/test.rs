// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use libra_schemadb::schema::assert_encode_decode;
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_encode_decode(
        address in any::<AccountAddress>(),
        seq_num in any::<u64>(),
        version in any::<Version>(),
    ) {
        assert_encode_decode::<TransactionByAccountSchema>(&(address, seq_num), &version);
    }
}
