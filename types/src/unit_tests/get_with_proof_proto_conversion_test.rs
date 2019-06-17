// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::get_with_proof::{
    RequestItem, ResponseItem, UpdateToLatestLedgerRequest, UpdateToLatestLedgerResponse,
};
use proptest::prelude::*;
use proto_conv::test_helper::assert_protobuf_encode_decode;

proptest! {
    #[test]
    fn test_update_to_latest_ledger_request(
        request in any::<UpdateToLatestLedgerRequest>()
    ) {
        assert_protobuf_encode_decode(&request);
    }

    #[test]
    fn test_request_item_conversion(item in any::<RequestItem>()) {
        assert_protobuf_encode_decode(&item);
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn test_response_item(item in any::<ResponseItem>()) {
        assert_protobuf_encode_decode(&item);
    }

    #[test]
    fn test_update_to_latest_ledger_response(
        response in any::<UpdateToLatestLedgerResponse>()
    ) {
        assert_protobuf_encode_decode(&response);
    }
}
