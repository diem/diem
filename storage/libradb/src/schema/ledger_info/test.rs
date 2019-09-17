// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use proptest::prelude::*;
use schemadb::schema::assert_encode_decode;
use types::crypto_proxies::LedgerInfoWithSignatures;

proptest! {
    #[test]
    fn test_encode_decode(
        ledger_info_with_sigs in any_with::<LedgerInfoWithSignatures>((1..10).into())
    ) {
        assert_encode_decode::<LedgerInfoSchema>(&0, &ledger_info_with_sigs);
    }
}
