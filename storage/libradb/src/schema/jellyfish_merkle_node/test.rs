// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use crypto::HashValue;
use jellyfish_merkle::node_type::Node;
use proptest::prelude::*;
use schemadb::schema::assert_encode_decode;
use types::account_state_blob::AccountStateBlob;

proptest! {
    #[test]
    fn test_jellyfish_merkle_node_schema(
        node_key in any::<NodeKey>(),
        account_key in any::<HashValue>(),
        blob in any::<AccountStateBlob>(),
    ) {
        assert_encode_decode::<JellyfishMerkleNodeSchema>(
            &node_key,
            &Node::new_leaf(account_key, blob),
        );
    }
}
