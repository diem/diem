// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_prost_test_helpers::assert_protobuf_encode_decode;
use libra_types::mempool_status::MempoolStatus;
use proptest::prelude::*;

proptest! {
    #[test]
    fn account_state_blob_proto_roundtrip(mempool_status in any::<MempoolStatus>()) {
        assert_protobuf_encode_decode::<crate::proto::types::MempoolStatus, MempoolStatus>(&mempool_status);
    }
}
