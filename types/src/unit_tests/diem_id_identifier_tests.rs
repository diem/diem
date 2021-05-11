// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    transaction::diem_id_identifier::{DiemIdUserIdentifier, DiemIdVaspDomainIdentifier, DiemIdParseError},
};

#[test]
fn test_invalid_user_identifier() {
    // Test valid domain
    let raw_identifier = "abcd1234";
    let identifier = DiemIdUserIdentifier::new(&raw_identifier);
    assert!(identifier.is_ok());

    // Test having 64 characters is valid
    let raw_identifier = "aaaaaaaaaabbbbbbbbbbccccccccccddddddddddeeeeeeeeeeffffffffff1234";
    let identifier = DiemIdUserIdentifier::new(&raw_identifier);
    assert!(identifier.is_ok());

    // Test using "-" character is invalid
    let raw_identifier = "abcd!!!1234";
    let identifier = DiemIdUserIdentifier::new(&raw_identifier);
    assert_eq!(
        identifier.unwrap_err(),
        DiemIdParseError,
    );
    // Test having 64 characters is invalid
    let raw_identifier = "aaaaaaaaaabbbbbbbbbbccccccccccddddddddddeeeeeeeeeeffffffffff12345";
    let identifier = DiemIdUserIdentifier::new(&raw_identifier);
    assert_eq!(
        identifier.unwrap_err(),
        DiemIdParseError,
    );
}

#[test]
fn test_invalid_vasp_domain_identifier() {
    // Test valid domain
    let raw_identifier = "diem";
    let identifier = DiemIdVaspDomainIdentifier::new(&raw_identifier);
    assert!(identifier.is_ok());

    // Test having 63 characters is valid
    let raw_identifier = "aaaaaaaaaabbbbbbbbbbccccccccccddddddddddeeeeeeeeeeffffffffff123";
    let identifier = DiemIdVaspDomainIdentifier::new(&raw_identifier);
    assert!(identifier.is_ok());

    // Test using "-" character is invalid
    let raw_identifier = "diem-domain";
    let identifier = DiemIdVaspDomainIdentifier::new(&raw_identifier);
    assert_eq!(
        identifier.unwrap_err(),
        DiemIdParseError,
    );
    // Test having 64 characters is invalid
    let raw_identifier = "aaaaaaaaaabbbbbbbbbbccccccccccddddddddddeeeeeeeeeeffffffffffgggg";
    let identifier = DiemIdVaspDomainIdentifier::new(&raw_identifier);
    assert_eq!(
        identifier.unwrap_err(),
        DiemIdParseError,
    );
}
