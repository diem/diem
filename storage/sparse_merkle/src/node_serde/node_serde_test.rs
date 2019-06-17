// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    nibble_path::NibblePath,
    node_type::{BranchNode, ExtensionNode, LeafNode},
};
use crypto::HashValue;
use serde_test::{assert_tokens, Token};

fn append_hashvalue_tokens(tokens: &mut Vec<Token>, hash: HashValue) {
    for i in 0..HashValue::LENGTH {
        tokens.push(Token::U8(hash[i]));
    }
}

#[test]
fn test_serde_branch_type() {
    let child1 = HashValue::random();
    let child2 = HashValue::random();
    let mut branch_node = BranchNode::default();
    branch_node.set_child(1, (child1, true));
    branch_node.set_child(2, (child2, true));
    let mut tokens = vec![];
    tokens.extend_from_slice(&[
        Token::Tuple { len: 3 },
        Token::U16(0b0110),
        Token::U16(0b0110),
        Token::Seq { len: Some(2) },
        Token::Struct {
            name: "HashValue",
            len: 1,
        },
        Token::Str("hash"),
        Token::Tuple { len: 32 },
    ]);
    append_hashvalue_tokens(&mut tokens, child1);
    tokens.extend_from_slice(&[
        Token::TupleEnd,
        Token::StructEnd,
        Token::Struct {
            name: "HashValue",
            len: 1,
        },
        Token::Str("hash"),
        Token::Tuple { len: 32 },
    ]);
    append_hashvalue_tokens(&mut tokens, child2);
    tokens.extend_from_slice(&[
        Token::TupleEnd,
        Token::StructEnd,
        Token::SeqEnd,
        Token::TupleEnd,
    ]);
    assert_tokens(&branch_node, &tokens)
}

#[test]
fn test_serde_extension_type() {
    let child = HashValue::random();
    let path = vec![0xff, 0x10];
    let extension_node = ExtensionNode::new(NibblePath::new_odd(path.clone()), child);
    let mut tokens = vec![];
    tokens.extend_from_slice(&[
        Token::Tuple { len: 2 },
        Token::Seq { len: Some(2) },
        Token::U8(path[0]),
        Token::U8(path[1]),
        Token::SeqEnd,
        Token::Struct {
            name: "HashValue",
            len: 1,
        },
        Token::Str("hash"),
        Token::Tuple { len: 32 },
    ]);
    append_hashvalue_tokens(&mut tokens, child);
    tokens.extend_from_slice(&[Token::TupleEnd, Token::StructEnd, Token::TupleEnd]);
    assert_tokens(&extension_node, &tokens);
}

#[test]
fn test_serde_leaf_type() {
    let leaf_node = LeafNode::new(HashValue::random(), HashValue::random());
    let mut tokens = vec![];
    tokens.extend_from_slice(&[
        Token::Tuple { len: 2 },
        Token::Struct {
            name: "HashValue",
            len: 1,
        },
        Token::Str("hash"),
        Token::Tuple { len: 32 },
    ]);
    append_hashvalue_tokens(&mut tokens, leaf_node.key());
    tokens.extend_from_slice(&[
        Token::TupleEnd,
        Token::StructEnd,
        Token::Struct {
            name: "HashValue",
            len: 1,
        },
        Token::Str("hash"),
        Token::Tuple { len: 32 },
    ]);
    append_hashvalue_tokens(&mut tokens, leaf_node.value_hash());
    tokens.extend_from_slice(&[Token::TupleEnd, Token::StructEnd, Token::TupleEnd]);
    assert_tokens(&leaf_node, &tokens);
}
