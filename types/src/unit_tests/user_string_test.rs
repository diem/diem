// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::user_string::{UserStr, UserString};
use canonical_serialization::test_helper::assert_canonical_encode_decode;
use proptest::prelude::*;
use serde_json;
use std::borrow::Borrow;

proptest! {
    #[test]
    fn string_user_string_roundtrip(s in ".*") {
        let us = UserString::new(s.as_str());
        prop_assert_eq!(us.len(), s.len());
        prop_assert_eq!(us.is_empty(), s.is_empty());
        prop_assert_eq!(us.as_str(), s.as_str());
        prop_assert_eq!(us.as_bytes(), s.as_bytes());
        let s2 = us.into_string();
        prop_assert_eq!(s, s2);
    }

    #[test]
    fn user_string_string_roundtrip(us in any::<UserString>()) {
        let s = us.clone().into_string();
        let us2 = UserString::new(s);
        prop_assert_eq!(us, us2);
    }

    #[test]
    fn user_string_user_str_equivalence(s in ".*") {
        let user_str = UserStr::new(&s);
        let user_string = UserString::new(s.clone());
        prop_assert_eq!(user_str, user_string.as_user_str());
        prop_assert_eq!(user_str, user_string.as_ref());
        prop_assert_eq!(user_str, user_string.borrow());
        prop_assert_eq!(user_str.to_owned(), user_string);
    }

    #[test]
    fn serde_json_roundtrip(us in any::<UserString>()) {
        let ser = serde_json::to_string(&us).expect("UserString should serialize correctly");
        let us2 = serde_json::from_str(&ser).expect("UserString should deserialize correctly");
        prop_assert_eq!(us, us2);
    }

    #[test]
    fn user_string_canonical_serialization(set in any::<UserString>()) {
        assert_canonical_encode_decode(&set);
    }
}

/// Ensure that UserString instances serialize into strings directly, with no wrapper.
#[test]
fn serde_serialize_no_wrapper() {
    let foobar = UserString::new("foobar");
    let s = serde_json::to_string(&foobar).expect("UserString should serialize correctly");
    assert_eq!(s, "\"foobar\"");
}
