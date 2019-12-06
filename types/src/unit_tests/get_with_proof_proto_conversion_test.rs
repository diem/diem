// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    get_with_proof::{
        RequestItem, ResponseItem, UpdateToLatestLedgerRequest, UpdateToLatestLedgerResponse,
    },
    proto,
};
use libra_prost_ext::test_helpers::assert_protobuf_encode_decode;
use proptest::prelude::*;
use std::convert::TryFrom;

proptest! {
    #[test]
    fn test_update_to_latest_ledger_request(
        request in any::<UpdateToLatestLedgerRequest>()
    ) {
        assert_protobuf_encode_decode::<crate::proto::types::UpdateToLatestLedgerRequest, UpdateToLatestLedgerRequest>(&request);
    }

    #[test]
    fn test_request_item_conversion(item in any::<RequestItem>()) {
        assert_protobuf_encode_decode::<crate::proto::types::RequestItem, RequestItem>(&item);
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn test_response_item(item in any::<ResponseItem>()) {
        assert_protobuf_encode_decode::<crate::proto::types::ResponseItem, ResponseItem>(&item);
    }

    #[test]
    fn test_update_to_latest_ledger_response(
        response in any::<UpdateToLatestLedgerResponse>()
    ) {
        assert_protobuf_encode_decode::<crate::proto::types::UpdateToLatestLedgerResponse, UpdateToLatestLedgerResponse>(&response);
    }
}

#[test]
fn request_item_is_none() {
    let proto = proto::types::RequestItem::default();

    let maybe_request_item = RequestItem::try_from(proto);
    assert!(maybe_request_item.is_err());
}

#[test]
fn response_item_is_none() {
    let proto = proto::types::ResponseItem::default();

    let maybe_response_item = ResponseItem::try_from(proto);
    assert!(maybe_response_item.is_err());
}
