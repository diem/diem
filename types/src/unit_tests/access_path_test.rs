// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    access_path::AccessPath,
    account_address::{AccountAddress, ADDRESS_LENGTH},
};
use proptest::prelude::*;
use proto_conv::{test_helper::assert_protobuf_encode_decode, FromProto, IntoProto};

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
