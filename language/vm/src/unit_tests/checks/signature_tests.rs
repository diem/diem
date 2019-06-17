// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    errors::VMStaticViolation,
    file_format::{SignatureToken, StructHandleIndex},
    SignatureTokenKind,
};

#[test]
fn test_sig_token_structure() {
    // Valid cases.
    let bool_token = SignatureToken::Bool;
    assert_eq!(bool_token.check_structure(), None);
    let struct_token = SignatureToken::Struct(StructHandleIndex::new(0));
    assert_eq!(struct_token.check_structure(), None);
    let ref_token = SignatureToken::Reference(Box::new(struct_token.clone()));
    assert_eq!(ref_token.check_structure(), None);
    let mut_ref_token = SignatureToken::MutableReference(Box::new(struct_token.clone()));
    assert_eq!(mut_ref_token.check_structure(), None);

    // Invalid cases.
    let ref_ref_token = SignatureToken::Reference(Box::new(ref_token.clone()));
    assert_eq!(
        ref_ref_token.check_structure(),
        Some(VMStaticViolation::InvalidSignatureToken(
            ref_ref_token.clone(),
            SignatureTokenKind::Reference,
            SignatureTokenKind::Reference,
        ))
    );
    let ref_mut_ref_token = SignatureToken::Reference(Box::new(mut_ref_token.clone()));
    assert_eq!(
        ref_mut_ref_token.check_structure(),
        Some(VMStaticViolation::InvalidSignatureToken(
            ref_mut_ref_token.clone(),
            SignatureTokenKind::Reference,
            SignatureTokenKind::MutableReference,
        ))
    );
}
