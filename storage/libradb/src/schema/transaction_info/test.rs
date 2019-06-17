// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use crypto::HashValue;
use schemadb::schema::assert_encode_decode;
use types::transaction::TransactionInfo;

#[test]
fn test_encode_decode() {
    let txn_info = TransactionInfo::new(
        HashValue::random(),
        HashValue::random(),
        HashValue::random(),
        7,
    );
    assert_encode_decode::<TransactionInfoSchema>(&0u64, &txn_info);
}
