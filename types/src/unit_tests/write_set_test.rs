// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::write_set::{WriteSet, WriteSetMut, WriteOp};
use canonical_serialization::{
    CanonicalDeserializer, CanonicalSerializer, SimpleDeserializer, SimpleSerializer,
};
use proptest::prelude::*;
use crate::access_path::AccessPath;
use crate::account_address::AccountAddress;

proptest! {
    #[test]
    fn write_set_roundtrip_canonical_serialization(write_set in any::<WriteSet>()) {
        let mut serializer = SimpleSerializer::<Vec<u8>>::new();
        serializer.encode_struct(&write_set).unwrap();
        let serialized_bytes = serializer.get_output();

        let mut deserializer = SimpleDeserializer::new(&serialized_bytes);
        let output: WriteSet = deserializer.decode_struct().unwrap();
        assert_eq!(write_set, output);
    }
}

#[test]
fn test_write_set_merge(){
    let account1 = AccountAddress::random();
    let account2 = AccountAddress::random();
    let account3 = AccountAddress::random();
    let access_path1 = AccessPath::new_for_account(account1);
    let access_path2 = AccessPath::new_for_account(account2);
    let access_path3 = AccessPath::new_for_account(account3);

    let vec = vec![
        (access_path1.clone(),WriteOp::Value(vec![1])),
        (access_path2.clone(),WriteOp::Value(vec![2]))
    ];
    let vec2 = vec![
        (access_path1.clone(),WriteOp::Value(vec![0])),
        (access_path2.clone(),WriteOp::Deletion),
        (access_path3.clone(),WriteOp::Value(vec![3]))
    ];
    let mut write_set1 = WriteSetMut::new(vec);
    let write_set2 = WriteSetMut::new(vec2);
    write_set1.merge_with(&write_set2);
    assert_eq!(write_set1.len(), 3);
    if let WriteOp::Value(a1_value) = write_set1.find_write_op_mut(&access_path1).unwrap() {
        assert_eq!(a1_value[0], 0);
    }else{
        panic!("unexpect write_op.")
    }
    debug_assert!(WriteOp::Deletion == write_set1.find_write_op_mut(&access_path2).unwrap().clone());
    if let WriteOp::Value(a3_value) = write_set1.find_write_op_mut(&access_path3).unwrap(){
        assert_eq!(a3_value[0], 3);
    }else{
        panic!("unexpect write_op.")
    }

}