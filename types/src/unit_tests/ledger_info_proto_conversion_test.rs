// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::ledger_info::{LedgerInfo, LedgerInfoWithSignatures};
use crypto::ed25519::*;
use proptest::prelude::*;
use proto_conv::test_helper::assert_protobuf_encode_decode;

proptest! {
    #[test]
    fn test_ledger_info(ledger_info in any::<LedgerInfo>()) {
        assert_protobuf_encode_decode(&ledger_info);
    }

    #[test]
    fn test_ledger_info_with_signatures(
        ledger_info_with_signatures in any_with::<LedgerInfoWithSignatures<Ed25519Signature>>((0..11).into())
    ) {
        assert_protobuf_encode_decode(&ledger_info_with_signatures);
    }
}

proptest! {
    // generating many key pairs are computationally heavy, limiting number of cases
    #![proptest_config(ProptestConfig::with_cases(10))]
    #[test]
    fn test_ledger_info_with_many_signatures(
        // 100 is the number we have in mind in real world, setting 200 to have a good chance of hitting it
        ledger_info_with_signatures in any_with::<LedgerInfoWithSignatures<Ed25519Signature>>((0..200).into())
    ) {
        assert_protobuf_encode_decode(&ledger_info_with_signatures);
    }
}
