// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    access::ModuleAccess,
    errors::VMStaticViolation,
    file_format::{CompiledModule, SignatureToken},
    views::{
        FieldDefinitionView, FunctionSignatureView, LocalsSignatureView, SignatureTokenView,
        TypeSignatureView, ViewInternals,
    },
    SignatureTokenKind,
};

pub trait SignatureCheck {
    fn check_signatures(&self) -> Vec<VMStaticViolation>;
}

impl<'a, T: ModuleAccess> SignatureCheck for FunctionSignatureView<'a, T> {
    fn check_signatures(&self) -> Vec<VMStaticViolation> {
        self.return_tokens()
            .filter_map(|token| token.check_structure())
            .chain(
                self.arg_tokens()
                    .filter_map(|token| token.check_structure()),
            )
            .collect()
    }
}

impl<'a, T: ModuleAccess> SignatureCheck for TypeSignatureView<'a, T> {
    fn check_signatures(&self) -> Vec<VMStaticViolation> {
        self.token().check_structure().into_iter().collect()
    }
}

impl<'a, T: ModuleAccess> SignatureCheck for LocalsSignatureView<'a, T> {
    fn check_signatures(&self) -> Vec<VMStaticViolation> {
        self.tokens()
            .filter_map(|token| token.check_structure())
            .collect()
    }
}

impl<'a> FieldDefinitionView<'a, CompiledModule> {
    /// Field definitions have additional constraints on signatures -- field signatures cannot be
    /// references or mutable references.
    pub fn check_signature_refs(&self) -> Option<VMStaticViolation> {
        let type_signature = self.type_signature();
        let token = type_signature.token();
        let kind = token.kind();
        match kind {
            SignatureTokenKind::Reference | SignatureTokenKind::MutableReference => Some(
                VMStaticViolation::InvalidFieldDefReference(token.as_inner().clone(), kind),
            ),
            SignatureTokenKind::Value => None,
        }
    }
}

impl<'a, T: ModuleAccess> SignatureTokenView<'a, T> {
    /// Check that this token is structurally correct.
    /// In particular, check that the token has a reference only at the top level.
    #[inline]
    pub fn check_structure(&self) -> Option<VMStaticViolation> {
        self.as_inner().check_structure()
    }
}

impl SignatureToken {
    // See SignatureTokenView::check_structure for more details.
    pub(crate) fn check_structure(&self) -> Option<VMStaticViolation> {
        use SignatureToken::*;

        let inner_token_opt = match self {
            Reference(token) => Some(token),
            MutableReference(token) => Some(token),
            Bool | U64 | String | ByteArray | Address | Struct(_) => None,
        };
        if let Some(inner_token) = inner_token_opt {
            if inner_token.is_reference() {
                return Some(VMStaticViolation::InvalidSignatureToken(
                    self.clone(),
                    self.kind(),
                    inner_token.kind(),
                ));
            }
        }
        None
    }
}
