// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use crypto::HashValue;
use schemadb::schema::assert_encode_decode;

#[test]
fn test_account_state_row() {
    assert_encode_decode::<AccountStateSchema>(
        &HashValue::random(),
        &vec![0x01, 0x02, 0x03].into(),
    );
}
