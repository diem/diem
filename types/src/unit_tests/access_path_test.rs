// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{access_path::AccessPath, account_address::AccountAddress};

#[test]
fn access_path_ord() {
    let ap1 = AccessPath {
        address: AccountAddress::new([1u8; AccountAddress::LENGTH]),
        path: b"/foo/b".to_vec(),
    };
    let ap2 = AccessPath {
        address: AccountAddress::new([1u8; AccountAddress::LENGTH]),
        path: b"/foo/c".to_vec(),
    };
    let ap3 = AccessPath {
        address: AccountAddress::new([1u8; AccountAddress::LENGTH]),
        path: b"/foo/c".to_vec(),
    };
    let ap4 = AccessPath {
        address: AccountAddress::new([2u8; AccountAddress::LENGTH]),
        path: b"/foo/a".to_vec(),
    };
    assert!(ap1 < ap2);
    assert_eq!(ap2, ap3);
    assert!(ap3 < ap4);
}
