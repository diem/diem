// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use proptest::collection::vec;
use proto_conv::test_helper::assert_protobuf_encode_decode;

fn hash_blob(blob: &[u8]) -> HashValue {
    let mut hasher = AccountStateBlobHasher::default();
    hasher.write(blob);
    hasher.finish()
}

proptest! {
    #[test]
    fn account_state_blob_roundtrip(account_state_blob in any::<AccountStateBlob>()) {
        assert_protobuf_encode_decode(&account_state_blob);
    }

    #[test]
    fn account_state_blob_hash(blob in vec(any::<u8>(), 1..100)) {
        prop_assert_eq!(hash_blob(&blob), AccountStateBlob::from(blob.clone()).hash());
    }

    #[test]
    fn account_state_with_proof(account_state_with_proof in any::<AccountStateWithProof>()) {
        assert_protobuf_encode_decode(&account_state_with_proof);
    }
}

#[test]
fn test_debug_does_not_panic() {
    format!("{:#?}", AccountStateBlob::from(vec![1u8, 2u8, 3u8]));
}
