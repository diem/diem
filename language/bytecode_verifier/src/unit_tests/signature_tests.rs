// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::signature::check_structure;
use types::vm_error::StatusCode;
use vm::file_format::{SignatureToken, StructHandleIndex};

#[test]
fn test_sig_token_structure() {
    // Valid cases.
    let bool_token = SignatureToken::Bool;
    assert_eq!(check_structure(&bool_token), None);
    let struct_token = SignatureToken::Struct(StructHandleIndex::new(0), vec![]);
    assert_eq!(check_structure(&struct_token), None);
    let ref_token = SignatureToken::Reference(Box::new(struct_token.clone()));
    assert_eq!(check_structure(&ref_token), None);
    let mut_ref_token = SignatureToken::MutableReference(Box::new(struct_token.clone()));
    assert_eq!(check_structure(&mut_ref_token), None);

    // Invalid cases.
    let ref_ref_token = SignatureToken::Reference(Box::new(ref_token.clone()));
    assert_eq!(
        check_structure(&ref_ref_token)
            .expect("Invalid sig token structure")
            .major_status,
        StatusCode::INVALID_SIGNATURE_TOKEN
    );
    let ref_mut_ref_token = SignatureToken::Reference(Box::new(mut_ref_token.clone()));
    assert_eq!(
        check_structure(&ref_mut_ref_token)
            .expect("Invalid sig token structure")
            .major_status,
        StatusCode::INVALID_SIGNATURE_TOKEN
    );
}
