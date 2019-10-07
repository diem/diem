// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use proptest::prelude::*;
use schemadb::schema::assert_encode_decode;

proptest! {
    #[test]
    fn test_encode_decode(txn in any::<Transaction>()) {
        assert_encode_decode::<TransactionSchema>(&0u64, &txn);
    }
}
