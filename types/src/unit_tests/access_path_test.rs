// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    access_path::AccessPath,
    account_address::{AccountAddress, ADDRESS_LENGTH},
};
use proptest::prelude::*;
use proto_conv::{test_helper::assert_protobuf_encode_decode, FromProto, IntoProto};
use crate::account_config::{account_struct_tag, coin_struct_tag};
use crate::access_path::DataPath;

#[test]
fn access_path_ord() {
    let ap1 = AccessPath {
        address: AccountAddress::new([1u8; ADDRESS_LENGTH]),
        path: b"/foo/b".to_vec(),
    };
    let ap2 = AccessPath {
        address: AccountAddress::new([1u8; ADDRESS_LENGTH]),
        path: b"/foo/c".to_vec(),
    };
    let ap3 = AccessPath {
        address: AccountAddress::new([1u8; ADDRESS_LENGTH]),
        path: b"/foo/c".to_vec(),
    };
    let ap4 = AccessPath {
        address: AccountAddress::new([2u8; ADDRESS_LENGTH]),
        path: b"/foo/a".to_vec(),
    };
    assert!(ap1 < ap2);
    assert_eq!(ap2, ap3);
    assert!(ap3 < ap4);
}

#[test]
fn test_access_path_protobuf_conversion() {
    let address = AccountAddress::new([1u8; ADDRESS_LENGTH]);
    let path = b"/foo/bar".to_vec();
    let ap = AccessPath {
        address,
        path: path.clone(),
    };
    let proto_ap = ap.clone().into_proto();
    assert_eq!(Vec::from(&address), proto_ap.get_address());
    assert_eq!(path, proto_ap.get_path());
    assert_eq!(AccessPath::from_proto(proto_ap).unwrap(), ap);
}

#[test]
fn test_access_path_protobuf_conversion_error() {
    let mut proto_ap = crate::proto::access_path::AccessPath::new();
    // Not a valid address.
    proto_ap.set_address(vec![0x12, 0x34]);
    proto_ap.set_path(b"/foo/bar".to_vec());
    assert!(AccessPath::from_proto(proto_ap).is_err());
}

proptest! {
    #[test]
    fn test_access_path_to_protobuf_roundtrip(access_path in any::<AccessPath>()) {
        assert_protobuf_encode_decode(&access_path);
    }
}

#[test]
fn test_access_path_resource_tag() {
    let access_path = AccessPath::new_for_account_resource(AccountAddress::random());
    println!("{:#?}", access_path);

    let resource_tag = access_path.resource_tag().unwrap();
    assert_eq!(resource_tag, account_struct_tag());
}


#[test]
fn test_access_path_data_path() {
    let access_path = AccessPath::new_for_account_resource(AccountAddress::random());

    let data_path = access_path.data_path().unwrap();
    assert_eq!(data_path.is_onchain_resource(), true);

    let account_address = AccountAddress::random();
    let other_address = AccountAddress::random();
    let off_chain_access_path = AccessPath::channel_resource_access_path(account_address, other_address, account_struct_tag());
    let off_chain_data_path = off_chain_access_path.data_path().unwrap();
    assert_eq!(off_chain_data_path.is_channel_resource(), true);
}

#[test]
fn test_access_path_for_a_special_address() {
    let account_address = AccountAddress::random();
    //this address contains b'/'
    let account_address2 = AccountAddress::from_hex_literal("0x805d16dca68907bc45cb742fe466d153479ea27708b95608b22cb6bdcfff895d").unwrap();
    let access_path = AccessPath::channel_resource_access_path(account_address, account_address2, coin_struct_tag());
    println!("{:?}", access_path);
    assert_eq!(access_path.data_path().unwrap(), DataPath::channel_resource_path(account_address2, coin_struct_tag()))
}