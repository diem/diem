// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use libra_types::transaction::SignedTransaction;
use proptest::prelude::*;
use schemadb::schema::assert_encode_decode;

proptest! {
    #[test]
    fn test_encode_decode(txn in any::<SignedTransaction>()) {
        assert_encode_decode::<SignedTransactionSchema>(&0u64, &txn);
    }
}
