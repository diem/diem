// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{CommitBlockRequest, CommitBlockResponse, ExecuteBlockRequest, ExecuteBlockResponse};
use proptest::prelude::*;
use proptest_helpers::with_stack_size;
use proto_conv::test_helper::assert_protobuf_encode_decode;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn test_execute_block_request_roundtrip(execute_block_request in any::<ExecuteBlockRequest>()) {
        assert_protobuf_encode_decode(&execute_block_request);
    }

    #[test]
    fn test_commit_block_request_roundtrip(commit_block_request in any::<CommitBlockRequest>()) {
        assert_protobuf_encode_decode(&commit_block_request);
    }
}

proptest! {
    #[test]
    fn test_commit_block_response_roundtrip(commit_block_response in any::<CommitBlockResponse>()) {
        assert_protobuf_encode_decode(&commit_block_response);
    }
}

#[test]
fn test_execute_block_response_roundtrip() {
    with_stack_size(4 * 1024 * 1024, || {
        proptest!(|(execute_block_response in any::<ExecuteBlockResponse>())| {
            assert_protobuf_encode_decode(&execute_block_response);
        })
    })
    .unwrap();
}
