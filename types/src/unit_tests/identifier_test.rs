// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::identifier::{IdentStr, Identifier, ALLOWED_IDENTIFIERS};
use crate::test_helpers::assert_canonical_encode_decode;
use once_cell::sync::Lazy;
use proptest::prelude::*;
use regex::Regex;
use serde_json;
use std::borrow::Borrow;

#[test]
fn valid_identifiers() {
    let valid_identifiers = [
        "foo",
        "FOO",
        "Foo",
        "foo0",
        "FOO_0",
        "_Foo1",
        "FOO2_",
        "foo_bar_baz",
        "_0",
        "__",
        "____________________",
        // TODO: <SELF> is an exception. It should be removed once CompiledScript goes away.
        "<SELF>",
    ];
    for identifier in &valid_identifiers {
        assert!(
            Identifier::is_valid(identifier),
            "Identifier '{}' should be valid",
            identifier
        );
    }
}

#[test]
fn invalid_identifiers() {
    let invalid_identifiers = [
        "",
        "_",
        "0",
        "01",
        "9876",
        "0foo",
        ":foo",
        "fo\\o",
        "fo/o",
        "foo.",
        "foo-bar",
        "foo\u{1f389}",
    ];
    for identifier in &invalid_identifiers {
        assert!(
            !Identifier::is_valid(identifier),
            "Identifier '{}' should be invalid",
            identifier
        );
    }
}

proptest! {
    #[test]
    fn invalid_identifiers_proptest(identifier in invalid_identifier_strategy()) {
        // This effectively checks that if a string doesn't match the ALLOWED_IDENTIFIERS regex, it
        // will be rejected by the is_valid validator. Note that the converse is checked by the
        // Arbitrary impl for Identifier.
        prop_assert!(!Identifier::is_valid(&identifier));
    }

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

fn invalid_identifier_strategy() -> impl Strategy<Value = String> {
    static ALLOWED_IDENTIFIERS_REGEX: Lazy<Regex> = Lazy::new(|| {
        // Need to add anchors to ensure the entire string is matched.
        Regex::new(&format!("^(?:{})$", ALLOWED_IDENTIFIERS)).unwrap()
    });

    ".*".prop_filter("Valid identifiers should not be generated", |s| {
        // Most strings won't match the regex above, so local rejects are OK.
        !ALLOWED_IDENTIFIERS_REGEX.is_match(s)
    })
}

/// Ensure that Identifier instances serialize into strings directly, with no wrapper.
#[test]
fn serde_serialize_no_wrapper() {
    let foobar = Identifier::new("foobar").expect("should parse correctly");
    let s = serde_json::to_string(&foobar).expect("Identifier should serialize correctly");
    assert_eq!(s, "\"foobar\"");
}
