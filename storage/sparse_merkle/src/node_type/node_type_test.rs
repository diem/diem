// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    nibble_path::NibblePath,
    node_type::{BranchNode, Children, Node, NodeDecodeError},
};
use crypto::{
    hash::{CryptoHash, SPARSE_MERKLE_PLACEHOLDER_HASH},
    HashValue,
};
use types::proof::{SparseMerkleInternalNode, SparseMerkleLeafNode};

fn hash_internal(left: HashValue, right: HashValue) -> HashValue {
    SparseMerkleInternalNode::new(left, right).hash()
}

fn hash_leaf(key: HashValue, value_hash: HashValue) -> HashValue {
    SparseMerkleLeafNode::new(key, value_hash).hash()
}

#[test]
fn test_encode_decode() {
    let leaf_node = Node::new_leaf(HashValue::random(), HashValue::random());

    let mut children = Children::default();
    children.insert(0, (leaf_node.hash(), true));

    let nodes = vec![
        Node::new_branch(children),
        Node::new_leaf(HashValue::random(), HashValue::random()),
        Node::new_extension(NibblePath::new(vec![1, 2, 3, 4]), HashValue::random()),
    ];
    for n in &nodes {
        let v = n.encode().unwrap();
        assert_eq!(*n, Node::decode(&v).unwrap());
    }
    // Error cases
    if let Err(e) = Node::decode(&[]) {
        assert_eq!(
            e.downcast::<NodeDecodeError>().unwrap(),
            NodeDecodeError::EmptyInput
        );
    }
    if let Err(e) = Node::decode(&[100]) {
        assert_eq!(
            e.downcast::<NodeDecodeError>().unwrap(),
            NodeDecodeError::UnknownTag { unknown_tag: 100 }
        );
    }
}

#[test]
fn test_leaf_hash() {
    {
        let address = HashValue::random();
        let value_hash = HashValue::random();
        let hash = hash_leaf(address, value_hash);
        let leaf_node = Node::new_leaf(address, value_hash);
        assert_eq!(leaf_node.hash(), hash);
    }
}

#[test]
fn test_extension_hash() {
    {
        let mut hash = HashValue::zero();
        let extension_node = Node::new_extension(NibblePath::new(vec![0b_0000_1111]), hash);

        for _ in 0..4 {
            hash = hash_internal(*SPARSE_MERKLE_PLACEHOLDER_HASH, hash);
        }
        for _ in 4..8 {
            hash = hash_internal(hash, *SPARSE_MERKLE_PLACEHOLDER_HASH);
        }
        assert_eq!(extension_node.hash(), hash);
    }
    {
        let mut hash = HashValue::random();
        let extension_node = Node::new_extension(NibblePath::new_odd(vec![0b_1110_0000]), hash);

        hash = hash_internal(hash, *SPARSE_MERKLE_PLACEHOLDER_HASH);
        for _ in 1..4 {
            hash = hash_internal(*SPARSE_MERKLE_PLACEHOLDER_HASH, hash);
        }
        assert_eq!(extension_node.hash(), hash);
    }
}

#[test]
fn test_branch_hash_and_proof() {
    // leaf case 1
    {
        let mut branch_node = BranchNode::new(Children::default());
        let hash1 = HashValue::random();
        let hash2 = HashValue::random();
        branch_node.set_child(4, (hash1, true));
        branch_node.set_child(15, (hash2, true));
        // Branch node will have a structure below
        //
        //              root
        //              / \
        //             /   \
        //        leaf1     leaf2
        //
        let root_hash = hash_internal(hash1, hash2);
        assert_eq!(branch_node.hash(), root_hash);

        for i in 0..8 {
            assert_eq!(
                branch_node.get_child_for_proof_and_siblings(i),
                (Some(hash1), vec![hash2])
            );
        }
        for i in 8..16 {
            assert_eq!(
                branch_node.get_child_for_proof_and_siblings(i),
                (Some(hash2), vec![hash1])
            );
        }
    }

    // leaf case 2
    {
        let mut branch_node = BranchNode::new(Children::default());
        let hash1 = HashValue::random();
        let hash2 = HashValue::random();
        branch_node.set_child(4, (hash1, true));
        branch_node.set_child(7, (hash2, true));
        // Branch node will have a structure below
        //
        //              root
        //              /
        //             /
        //            x2
        //             \
        //              \
        //               x1
        //              / \
        //             /   \
        //        leaf1     leaf2

        let hash_x1 = hash_internal(hash1, hash2);
        let hash_x2 = hash_internal(*SPARSE_MERKLE_PLACEHOLDER_HASH, hash_x1);

        let root_hash = hash_internal(hash_x2, *SPARSE_MERKLE_PLACEHOLDER_HASH);
        assert_eq!(branch_node.hash(), root_hash);

        for i in 0..4 {
            assert_eq!(
                branch_node.get_child_for_proof_and_siblings(i),
                (None, vec![*SPARSE_MERKLE_PLACEHOLDER_HASH, hash_x1])
            );
        }

        for i in 4..6 {
            assert_eq!(
                branch_node.get_child_for_proof_and_siblings(i),
                (
                    Some(hash1),
                    vec![
                        *SPARSE_MERKLE_PLACEHOLDER_HASH,
                        *SPARSE_MERKLE_PLACEHOLDER_HASH,
                        hash2
                    ]
                )
            );
        }

        for i in 6..8 {
            assert_eq!(
                branch_node.get_child_for_proof_and_siblings(i),
                (
                    Some(hash2),
                    vec![
                        *SPARSE_MERKLE_PLACEHOLDER_HASH,
                        *SPARSE_MERKLE_PLACEHOLDER_HASH,
                        hash1
                    ]
                )
            );
        }

        for i in 8..16 {
            assert_eq!(
                branch_node.get_child_for_proof_and_siblings(i),
                (None, vec![hash_x2])
            );
        }
    }

    // leaf case 3
    {
        let mut branch_node = BranchNode::new(Children::default());
        let hash1 = HashValue::random();
        let hash2 = HashValue::random();
        let hash3 = HashValue::random();
        branch_node.set_child(0, (hash1, true));
        branch_node.set_child(7, (hash2, true));
        branch_node.set_child(8, (hash3, true));
        // Branch node will have a structure below
        //
        //               root
        //               / \
        //              /   \
        //             x     leaf3
        //            / \
        //           /   \
        //      leaf1     leaf2
        //

        let hash_x = hash_internal(hash1, hash2);
        let root_hash = hash_internal(hash_x, hash3);

        for i in 0..4 {
            assert_eq!(
                branch_node.get_child_for_proof_and_siblings(i),
                (Some(hash1), vec![hash3, hash2])
            );
        }

        for i in 4..8 {
            assert_eq!(
                branch_node.get_child_for_proof_and_siblings(i),
                (Some(hash2), vec![hash3, hash1])
            );
        }

        for i in 8..16 {
            assert_eq!(
                branch_node.get_child_for_proof_and_siblings(i),
                (Some(hash3), vec![hash_x])
            );
        }

        assert_eq!(branch_node.hash(), root_hash);
    }

    // non-leaf case 1
    {
        let mut branch_node = BranchNode::new(Children::default());
        let hash1 = HashValue::random();
        let hash2 = HashValue::random();
        branch_node.set_child(4, (hash1, false));
        branch_node.set_child(15, (hash2, false));
        // Branch node (B) will have a structure below
        //
        //              root
        //              / \
        //             /   \
        //            x3    x6
        //             \     \
        //              \     \
        //              x2     x5
        //              /       \
        //             /         \
        //            x1          x4
        //           /             \
        //          /               \
        // non-leaf1                 non-leaf2
        //
        let hash_x1 = hash_internal(hash1, *SPARSE_MERKLE_PLACEHOLDER_HASH);
        let hash_x2 = hash_internal(hash_x1, *SPARSE_MERKLE_PLACEHOLDER_HASH);
        let hash_x3 = hash_internal(*SPARSE_MERKLE_PLACEHOLDER_HASH, hash_x2);
        let hash_x4 = hash_internal(*SPARSE_MERKLE_PLACEHOLDER_HASH, hash2);
        let hash_x5 = hash_internal(*SPARSE_MERKLE_PLACEHOLDER_HASH, hash_x4);
        let hash_x6 = hash_internal(*SPARSE_MERKLE_PLACEHOLDER_HASH, hash_x5);
        let root_hash = hash_internal(hash_x3, hash_x6);
        assert_eq!(branch_node.hash(), root_hash);

        for i in 0..4 {
            assert_eq!(
                branch_node.get_child_for_proof_and_siblings(i),
                (None, vec![hash_x6, hash_x2])
            );
        }

        assert_eq!(
            branch_node.get_child_for_proof_and_siblings(4),
            (
                Some(hash1),
                vec![
                    hash_x6,
                    *SPARSE_MERKLE_PLACEHOLDER_HASH,
                    *SPARSE_MERKLE_PLACEHOLDER_HASH,
                    *SPARSE_MERKLE_PLACEHOLDER_HASH
                ]
            )
        );

        assert_eq!(
            branch_node.get_child_for_proof_and_siblings(5),
            (
                None,
                vec![
                    hash_x6,
                    *SPARSE_MERKLE_PLACEHOLDER_HASH,
                    *SPARSE_MERKLE_PLACEHOLDER_HASH,
                    hash1
                ]
            )
        );
        for i in 6..8 {
            assert_eq!(
                branch_node.get_child_for_proof_and_siblings(i),
                (
                    None,
                    vec![hash_x6, *SPARSE_MERKLE_PLACEHOLDER_HASH, hash_x1]
                )
            );
        }

        for i in 8..12 {
            assert_eq!(
                branch_node.get_child_for_proof_and_siblings(i),
                (None, vec![hash_x3, hash_x5])
            );
        }

        for i in 12..14 {
            assert_eq!(
                branch_node.get_child_for_proof_and_siblings(i),
                (
                    None,
                    vec![hash_x3, *SPARSE_MERKLE_PLACEHOLDER_HASH, hash_x4]
                )
            );
        }
        assert_eq!(
            branch_node.get_child_for_proof_and_siblings(14),
            (
                None,
                vec![
                    hash_x3,
                    *SPARSE_MERKLE_PLACEHOLDER_HASH,
                    *SPARSE_MERKLE_PLACEHOLDER_HASH,
                    hash2
                ]
            )
        );
        assert_eq!(
            branch_node.get_child_for_proof_and_siblings(15),
            (
                Some(hash2),
                vec![
                    hash_x3,
                    *SPARSE_MERKLE_PLACEHOLDER_HASH,
                    *SPARSE_MERKLE_PLACEHOLDER_HASH,
                    *SPARSE_MERKLE_PLACEHOLDER_HASH
                ]
            )
        );
    }

    // non-leaf case 2
    {
        let mut branch_node = BranchNode::new(Children::default());
        let hash1 = HashValue::random();
        let hash2 = HashValue::random();
        branch_node.set_child(0, (hash1, false));
        branch_node.set_child(7, (hash2, false));
        // Branch node will have a structure below
        //
        //                     root
        //                     /
        //                    /
        //                   x5
        //                  / \
        //                 /   \
        //               x2     x4
        //               /       \
        //              /         \
        //            x1           x3
        //            /             \
        //           /               \
        //  non-leaf1                 non-leaf2

        let hash_x1 = hash_internal(hash1, *SPARSE_MERKLE_PLACEHOLDER_HASH);
        let hash_x2 = hash_internal(hash_x1, *SPARSE_MERKLE_PLACEHOLDER_HASH);
        let hash_x3 = hash_internal(*SPARSE_MERKLE_PLACEHOLDER_HASH, hash2);
        let hash_x4 = hash_internal(*SPARSE_MERKLE_PLACEHOLDER_HASH, hash_x3);
        let hash_x5 = hash_internal(hash_x2, hash_x4);
        let root_hash = hash_internal(hash_x5, *SPARSE_MERKLE_PLACEHOLDER_HASH);
        assert_eq!(branch_node.hash(), root_hash);

        assert_eq!(
            branch_node.get_child_for_proof_and_siblings(0),
            (
                Some(hash1),
                vec![
                    *SPARSE_MERKLE_PLACEHOLDER_HASH,
                    hash_x4,
                    *SPARSE_MERKLE_PLACEHOLDER_HASH,
                    *SPARSE_MERKLE_PLACEHOLDER_HASH,
                ]
            )
        );

        assert_eq!(
            branch_node.get_child_for_proof_and_siblings(1),
            (
                None,
                vec![
                    *SPARSE_MERKLE_PLACEHOLDER_HASH,
                    hash_x4,
                    *SPARSE_MERKLE_PLACEHOLDER_HASH,
                    hash1,
                ]
            )
        );

        for i in 2..4 {
            assert_eq!(
                branch_node.get_child_for_proof_and_siblings(i),
                (
                    None,
                    vec![*SPARSE_MERKLE_PLACEHOLDER_HASH, hash_x4, hash_x1]
                )
            );
        }

        for i in 4..6 {
            assert_eq!(
                branch_node.get_child_for_proof_and_siblings(i),
                (
                    None,
                    vec![*SPARSE_MERKLE_PLACEHOLDER_HASH, hash_x2, hash_x3]
                )
            );
        }

        assert_eq!(
            branch_node.get_child_for_proof_and_siblings(6),
            (
                None,
                vec![
                    *SPARSE_MERKLE_PLACEHOLDER_HASH,
                    hash_x2,
                    *SPARSE_MERKLE_PLACEHOLDER_HASH,
                    hash2
                ]
            )
        );

        assert_eq!(
            branch_node.get_child_for_proof_and_siblings(7),
            (
                Some(hash2),
                vec![
                    *SPARSE_MERKLE_PLACEHOLDER_HASH,
                    hash_x2,
                    *SPARSE_MERKLE_PLACEHOLDER_HASH,
                    *SPARSE_MERKLE_PLACEHOLDER_HASH,
                ]
            )
        );

        for i in 8..16 {
            assert_eq!(
                branch_node.get_child_for_proof_and_siblings(i),
                (None, vec![hash_x5])
            );
        }
    }

    // mixed case
    {
        let mut branch_node = BranchNode::new(Children::default());
        let hash1 = HashValue::random();
        let hash2 = HashValue::random();
        let hash3 = HashValue::random();
        branch_node.set_child(0, (hash1, true));
        branch_node.set_child(2, (hash2, false));
        branch_node.set_child(7, (hash3, false));
        // Branch node (B) will have a structure below
        //
        //               B (root hash)
        //              /
        //             /
        //            x5
        //           / \
        //          /   \
        //         x2    x4
        //        / \     \
        //       /   \     \
        //  leaf1    x1     x3
        //           /       \
        //          /         \
        //   non-leaf2         non-leaf3
        //
        let hash_x1 = hash_internal(hash2, *SPARSE_MERKLE_PLACEHOLDER_HASH);
        let hash_x2 = hash_internal(hash1, hash_x1);
        let hash_x3 = hash_internal(*SPARSE_MERKLE_PLACEHOLDER_HASH, hash3);
        let hash_x4 = hash_internal(*SPARSE_MERKLE_PLACEHOLDER_HASH, hash_x3);
        let hash_x5 = hash_internal(hash_x2, hash_x4);
        let root_hash = hash_internal(hash_x5, *SPARSE_MERKLE_PLACEHOLDER_HASH);
        assert_eq!(branch_node.hash(), root_hash);

        for i in 0..2 {
            assert_eq!(
                branch_node.get_child_for_proof_and_siblings(i),
                (
                    Some(hash1),
                    vec![*SPARSE_MERKLE_PLACEHOLDER_HASH, hash_x4, hash_x1]
                )
            );
        }

        assert_eq!(
            branch_node.get_child_for_proof_and_siblings(2),
            (
                Some(hash2),
                vec![
                    *SPARSE_MERKLE_PLACEHOLDER_HASH,
                    hash_x4,
                    hash1,
                    *SPARSE_MERKLE_PLACEHOLDER_HASH,
                ]
            )
        );

        assert_eq!(
            branch_node.get_child_for_proof_and_siblings(3),
            (
                None,
                vec![*SPARSE_MERKLE_PLACEHOLDER_HASH, hash_x4, hash1, hash2,]
            )
        );

        for i in 4..6 {
            assert_eq!(
                branch_node.get_child_for_proof_and_siblings(i),
                (
                    None,
                    vec![*SPARSE_MERKLE_PLACEHOLDER_HASH, hash_x2, hash_x3]
                )
            );
        }

        assert_eq!(
            branch_node.get_child_for_proof_and_siblings(6),
            (
                None,
                vec![
                    *SPARSE_MERKLE_PLACEHOLDER_HASH,
                    hash_x2,
                    *SPARSE_MERKLE_PLACEHOLDER_HASH,
                    hash3,
                ]
            )
        );

        assert_eq!(
            branch_node.get_child_for_proof_and_siblings(7),
            (
                Some(hash3),
                vec![
                    *SPARSE_MERKLE_PLACEHOLDER_HASH,
                    hash_x2,
                    *SPARSE_MERKLE_PLACEHOLDER_HASH,
                    *SPARSE_MERKLE_PLACEHOLDER_HASH,
                ]
            )
        );

        for i in 8..16 {
            assert_eq!(
                branch_node.get_child_for_proof_and_siblings(i),
                (None, vec![hash_x5])
            );
        }
    }
}
