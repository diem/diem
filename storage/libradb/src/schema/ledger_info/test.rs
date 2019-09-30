// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use libra_schemadb::schema::assert_encode_decode;
use libra_types::crypto_proxies::LedgerInfoWithSignatures;
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_encode_decode(
        ledger_info_with_sigs in any_with::<LedgerInfoWithSignatures>((1..10).into())
    ) {
        assert_encode_decode::<LedgerInfoSchema>(&0, &ledger_info_with_sigs);
    }
}
