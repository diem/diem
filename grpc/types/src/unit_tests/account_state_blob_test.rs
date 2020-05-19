// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_prost_test_helpers::assert_protobuf_encode_decode;
use libra_types::account_state_blob::{AccountStateBlob, AccountStateWithProof};
use proptest::prelude::*;

proptest! {
    #[test]
    fn account_state_blob_proto_roundtrip(account_state_blob in any::<AccountStateBlob>()) {
        assert_protobuf_encode_decode::<crate::proto::types::AccountStateBlob, AccountStateBlob>(&account_state_blob);
    }

    #[test]
    fn account_state_with_proof_proto_roundtrip(account_state_with_proof in any::<AccountStateWithProof>()) {
        assert_protobuf_encode_decode::<crate::proto::types::AccountStateWithProof, AccountStateWithProof>(&account_state_with_proof);
    }
}
