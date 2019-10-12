// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::vm_string::{VMStr, VMString};
use proptest::prelude::*;
use serde_json;
use std::borrow::Borrow;

proptest! {
    #[test]
    fn string_vm_string_roundtrip(s in ".*") {
        let us = VMString::new(s.as_str());
        prop_assert_eq!(us.len(), s.len());
        prop_assert_eq!(us.is_empty(), s.is_empty());
        prop_assert_eq!(us.as_str(), s.as_str());
        prop_assert_eq!(us.as_bytes(), s.as_bytes());
        let s2 = us.into_string();
        prop_assert_eq!(s, s2);
    }

    #[test]
    fn vm_string_string_roundtrip(us in any::<VMString>()) {
        let s = us.clone().into_string();
        let us2 = VMString::new(s);
        prop_assert_eq!(us, us2);
    }

    #[test]
    fn vm_string_vm_str_equivalence(s in ".*") {
        let vm_str = VMStr::new(&s);
        let vm_string = VMString::new(s.clone());
        prop_assert_eq!(vm_str, vm_string.as_vm_str());
        prop_assert_eq!(vm_str, vm_string.as_ref());
        prop_assert_eq!(vm_str, vm_string.borrow());
        prop_assert_eq!(vm_str.to_owned(), vm_string);
    }

    #[test]
    fn serde_json_roundtrip(us in any::<VMString>()) {
        let ser = serde_json::to_string(&us).expect("VMString should serialize correctly");
        let us2 = serde_json::from_str(&ser).expect("VMString should deserialize correctly");
        prop_assert_eq!(us, us2);
    }

    #[test]
    fn vm_string_canonical_serialization(set in any::<VMString>()) {
        let bytes = lcs::to_bytes(&set).unwrap();
        let s: VMString = lcs::from_bytes(&bytes).unwrap();
        assert_eq!(set, s);
    }
}

/// Ensure that VMString instances serialize into strings directly, with no wrapper.
#[test]
fn serde_serialize_no_wrapper() {
    let foobar = VMString::new("foobar");
    let s = serde_json::to_string(&foobar).expect("VMString should serialize correctly");
    assert_eq!(s, "\"foobar\"");
}
