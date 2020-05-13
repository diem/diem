// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_prost_ext::test_helpers::assert_protobuf_encode_decode;
use libra_types::{access_path::AccessPath, account_address::AccountAddress};
use proptest::prelude::*;
use std::convert::TryFrom;

#[test]
fn test_access_path_protobuf_conversion() {
    let address = AccountAddress::new([1u8; AccountAddress::LENGTH]);
    let path = b"/foo/bar".to_vec();
    let ap = AccessPath {
        address,
        path: path.clone(),
    };
    let proto_ap: crate::proto::types::AccessPath = ap.clone().into();
    assert_eq!(Vec::from(&address), proto_ap.address);
    assert_eq!(path, proto_ap.path);
    assert_eq!(AccessPath::try_from(proto_ap).unwrap(), ap);
}

#[test]
fn test_access_path_protobuf_conversion_error() {
    let mut proto_ap = crate::proto::types::AccessPath::default();
    // Not a valid address.
    proto_ap.address = vec![0x12, 0x34];
    proto_ap.path = b"/foo/bar".to_vec();
    assert!(AccessPath::try_from(proto_ap).is_err());
}

proptest! {
    #[test]
    fn test_access_path_to_protobuf_roundtrip(access_path in any::<AccessPath>()) {
        assert_protobuf_encode_decode::<crate::proto::types::AccessPath, AccessPath>(&access_path);
    }
}
