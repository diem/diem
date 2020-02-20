// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    get_with_proof::{
        RequestItem, ResponseItem, UpdateToLatestLedgerRequest, UpdateToLatestLedgerResponse,
    },
    proto,
    test_helpers::assert_canonical_encode_decode,
};
use libra_prost_ext::test_helpers::assert_protobuf_encode_decode;
use proptest::prelude::*;
use std::convert::TryFrom;

proptest! {
    #[test]
    fn update_to_latest_ledger_request_proto_roundtrip(
        request in any::<UpdateToLatestLedgerRequest>()
    ) {
        assert_protobuf_encode_decode::<crate::proto::types::UpdateToLatestLedgerRequest, UpdateToLatestLedgerRequest>(&request);
    }

    fn update_to_latest_ledger_request_lcs_roundtrip(
        request in any::<UpdateToLatestLedgerRequest>()
    ) {
        assert_canonical_encode_decode(request);
    }

    #[test]
    fn request_item_proto_roundtrip(item in any::<RequestItem>()) {
        assert_protobuf_encode_decode::<crate::proto::types::RequestItem, RequestItem>(&item);
    }

    #[test]
    fn request_item_lcs_roundtrip(item in any::<RequestItem>()) {
        assert_canonical_encode_decode(item);
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn update_to_latest_ledger_response_proto_roundtrip(
        response in any::<UpdateToLatestLedgerResponse>()
    ) {
        assert_protobuf_encode_decode::<crate::proto::types::UpdateToLatestLedgerResponse, UpdateToLatestLedgerResponse>(&response);
    }

    #[test]
    fn update_to_latest_ledger_response_lcs_roundtrip(
        response in any::<UpdateToLatestLedgerResponse>()
    ) {
        assert_canonical_encode_decode(response);
    }

    #[test]
    fn response_item_proto_roundtrip(item in any::<ResponseItem>()) {
        assert_protobuf_encode_decode::<crate::proto::types::ResponseItem, ResponseItem>(&item);
    }

    #[test]
    fn response_item_lcs_roundtrip(item in any::<ResponseItem>()) {
        assert_canonical_encode_decode(item);
    }

}

#[test]
fn proto_request_item_is_none() {
    let proto = proto::types::RequestItem::default();

    let maybe_request_item = RequestItem::try_from(proto);
    assert!(maybe_request_item.is_err());
}

#[test]
fn proto_response_item_is_none() {
    let proto = proto::types::ResponseItem::default();

    let maybe_response_item = ResponseItem::try_from(proto);
    assert!(maybe_response_item.is_err());
}
