// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::channel::channel_participant_struct_tag;
use crate::test_helpers::assert_canonical_encode_decode;
use crate::{
    access_path::AccessPath,
    account_address::AccountAddress,
    write_set::{WriteOp, WriteSet, WriteSetMut},
};
use proptest::prelude::*;

proptest! {
    #[test]
    fn write_set_roundtrip_canonical_serialization(write_set in any::<WriteSet>()) {
        assert_canonical_encode_decode(write_set);
    }
}

#[test]
fn test_write_set_merge() {
    let account1 = AccountAddress::random();
    let account2 = AccountAddress::random();
    let account3 = AccountAddress::random();
    let access_path1 = AccessPath::new_for_account(account1);
    let access_path2 = AccessPath::new_for_account(account2);
    let access_path3 = AccessPath::new_for_account(account3);

    let vec = vec![
        (access_path1.clone(), WriteOp::Value(vec![1])),
        (access_path2.clone(), WriteOp::Value(vec![2])),
    ];
    let vec2 = vec![
        (access_path1.clone(), WriteOp::Value(vec![0])),
        (access_path2.clone(), WriteOp::Deletion),
        (access_path3.clone(), WriteOp::Value(vec![3])),
    ];
    let mut write_set1 = WriteSetMut::new(vec);
    let write_set2 = WriteSetMut::new(vec2);
    write_set1.merge_with(&write_set2);
    assert_eq!(write_set1.len(), 3);
    if let WriteOp::Value(a1_value) = write_set1.find_write_op_mut(&access_path1).unwrap() {
        assert_eq!(a1_value[0], 0);
    } else {
        panic!("unexpect write_op.")
    }
    debug_assert!(
        WriteOp::Deletion == write_set1.find_write_op_mut(&access_path2).unwrap().clone()
    );
    if let WriteOp::Value(a3_value) = write_set1.find_write_op_mut(&access_path3).unwrap() {
        assert_eq!(a3_value[0], 3);
    } else {
        panic!("unexpect write_op.")
    }
}

#[test]
fn test_contains_channel_resource() {
    let account1 = AccountAddress::random();
    let account2 = AccountAddress::random();
    let channel_address = AccountAddress::channel_address(vec![account1, account2].as_slice());

    let vec = vec![
        (
            AccessPath::channel_resource_access_path(
                channel_address,
                account1,
                channel_participant_struct_tag(),
            ),
            WriteOp::Value(vec![1]),
        ),
        (
            AccessPath::channel_resource_access_path(
                channel_address,
                account2,
                channel_participant_struct_tag(),
            ),
            WriteOp::Value(vec![2]),
        ),
    ];
    let write_set = WriteSetMut::new(vec).freeze().unwrap();
    assert!(write_set.contains_channel_resource(&account1));
    assert!(!write_set.contains_channel_resource(&AccountAddress::random()));

    let vec2 = vec![(
        AccessPath::new_for_account(account1),
        WriteOp::Value(vec![1]),
    )];
    let write_set2 = WriteSetMut::new(vec2).freeze().unwrap();
    assert!(!write_set2.contains_channel_resource(&account1));
}
