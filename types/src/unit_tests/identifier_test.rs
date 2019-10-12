// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::identifier::{IdentStr, Identifier};
use crate::test_helpers::assert_canonical_encode_decode;
use proptest::prelude::*;
use serde_json;
use std::borrow::Borrow;

proptest! {
    #[test]
    fn identifier_string_roundtrip(identifier in any::<Identifier>()) {
        let s = identifier.clone().into_string();
        let id2 = Identifier::new(s).expect("identifier should parse correctly");
        prop_assert_eq!(identifier, id2);
    }

    #[test]
    fn identifier_ident_str_equivalence(identifier in any::<Identifier>()) {
        let s = identifier.clone().into_string();
        let ident_str = IdentStr::new(&s).expect("identifier should parse correctly");
        prop_assert_eq!(ident_str, identifier.as_ident_str());
        prop_assert_eq!(ident_str, identifier.as_ref());
        prop_assert_eq!(ident_str, identifier.borrow());
        prop_assert_eq!(ident_str.to_owned(), identifier);
    }

    #[test]
    fn serde_json_roundtrip(identifier in any::<Identifier>()) {
        let ser = serde_json::to_string(&identifier).expect("should serialize correctly");
        let id2 = serde_json::from_str(&ser).expect("should deserialize correctly");
        prop_assert_eq!(identifier, id2);
    }

    #[test]
    fn identifier_canonical_serialization(identifier in any::<Identifier>()) {
        assert_canonical_encode_decode(identifier);
    }
}

/// Ensure that UserString instances serialize into strings directly, with no wrapper.
#[test]
fn serde_serialize_no_wrapper() {
    let foobar = Identifier::new("foobar").expect("should parse correctly");
    let s = serde_json::to_string(&foobar).expect("UserString should serialize correctly");
    assert_eq!(s, "\"foobar\"");
}
