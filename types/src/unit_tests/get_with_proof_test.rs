// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::get_with_proof::{
    RequestItem, ResponseItem, UpdateToLatestLedgerRequest, UpdateToLatestLedgerResponse,
};
use lcs::test_helpers::assert_canonical_encode_decode;
use proptest::prelude::*;

proptest! {

    fn update_to_latest_ledger_request_lcs_roundtrip(
        request in any::<UpdateToLatestLedgerRequest>()
    ) {
        assert_canonical_encode_decode(request);
    }


    #[test]
    fn request_item_lcs_roundtrip(item in any::<RequestItem>()) {
        assert_canonical_encode_decode(item);
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]


    #[test]
    fn update_to_latest_ledger_response_lcs_roundtrip(
        response in any::<UpdateToLatestLedgerResponse>()
    ) {
        assert_canonical_encode_decode(response);
    }

    #[test]
    fn response_item_lcs_roundtrip(item in any::<ResponseItem>()) {
        assert_canonical_encode_decode(item);
    }

}
