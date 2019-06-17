// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use schemadb::schema::assert_encode_decode;

#[test]
fn test_single_entry_schema() {
    assert_encode_decode::<SingleEntrySchema>(
        &SingleEntryKey::ConsensusState,
        &vec![1u8, 2u8, 3u8],
    );
}
