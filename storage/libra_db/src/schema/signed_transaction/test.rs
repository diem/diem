// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use libra_schemadb::schema::assert_encode_decode;
use libra_types::transaction::SignedTransaction;
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_encode_decode(txn in any::<SignedTransaction>()) {
        assert_encode_decode::<SignedTransactionSchema>(&0u64, &txn);
    }
}
