// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module implements a checker for verifying signature tokens used in types of function
//! parameters, locals, and fields of structs are well-formed. References can only occur at the
//! top-level in all tokens.  Additionally, references cannot occur at all in field types.
use vm::{
    access::ModuleAccess,
    errors::{VMStaticViolation, VerificationError},
    file_format::{CompiledModule, SignatureToken},
    views::{
        FieldDefinitionView, FunctionSignatureView, LocalsSignatureView, ModuleView,
        TypeSignatureView, ViewInternals,
    },
    IndexKind, SignatureTokenKind,
};

pub struct SignatureChecker<'a> {
    module_view: ModuleView<'a, CompiledModule>,
}

impl<'a> SignatureChecker<'a> {
    pub fn new(module: &'a CompiledModule) -> Self {
        Self {
            module_view: ModuleView::new(module),
        }
    }

    pub fn verify(self) -> Vec<VerificationError> {
        let mut errors: Vec<Vec<_>> = vec![];

        errors.push(Self::verify_impl(
            IndexKind::TypeSignature,
            self.module_view.type_signatures(),
        ));
        errors.push(Self::verify_impl(
            IndexKind::FunctionSignature,
            self.module_view.function_signatures(),
        ));
        errors.push(Self::verify_impl(
            IndexKind::LocalsSignature,
            self.module_view.locals_signatures(),
        ));

        let signature_ref_errors = self
            .module_view
            .fields()
            .enumerate()
            .filter_map(move |(idx, view)| {
                check_signature_refs(&view).map(move |err| VerificationError {
                    kind: IndexKind::FieldDefinition,
                    idx,
                    err,
                })
            })
            .collect();
        errors.push(signature_ref_errors);

        errors.into_iter().flatten().collect()
    }

    #[inline]
    fn verify_impl(
        kind: IndexKind,
        views: impl Iterator<Item = impl SignatureCheck>,
    ) -> Vec<VerificationError> {
        views
            .enumerate()
            .map(move |(idx, view)| {
                view.check_signatures()
                    .into_iter()
                    .map(move |err| VerificationError { kind, idx, err })
            })
            .flatten()
            .collect()
    }
}

trait SignatureCheck {
    fn check_signatures(&self) -> Vec<VMStaticViolation>;
}

impl<'a, T: ModuleAccess> SignatureCheck for FunctionSignatureView<'a, T> {
    fn check_signatures(&self) -> Vec<VMStaticViolation> {
        self.return_tokens()
            .filter_map(|token| check_structure(token.as_inner()))
            .chain(
                self.arg_tokens()
                    .filter_map(|token| check_structure(token.as_inner())),
            )
            .collect()
    }
}

impl<'a, T: ModuleAccess> SignatureCheck for TypeSignatureView<'a, T> {
    fn check_signatures(&self) -> Vec<VMStaticViolation> {
        check_structure(self.token().as_inner())
            .into_iter()
            .collect()
    }
}

impl<'a, T: ModuleAccess> SignatureCheck for LocalsSignatureView<'a, T> {
    fn check_signatures(&self) -> Vec<VMStaticViolation> {
        self.tokens()
            .filter_map(|token| check_structure(token.as_inner()))
            .collect()
    }
}

/// Field definitions have additional constraints on signatures -- field signatures cannot be
/// references or mutable references.
pub(crate) fn check_signature_refs(
    view: &FieldDefinitionView<'_, CompiledModule>,
) -> Option<VMStaticViolation> {
    let type_signature = view.type_signature();
    let token = type_signature.token();
    let kind = token.kind();
    match kind {
        SignatureTokenKind::Reference | SignatureTokenKind::MutableReference => Some(
            VMStaticViolation::InvalidFieldDefReference(token.as_inner().clone(), kind),
        ),
        SignatureTokenKind::Value => None,
    }
}

/// Check that this token is structurally correct.
/// In particular, check that the token has a reference only at the top level.
pub(crate) fn check_structure(token: &SignatureToken) -> Option<VMStaticViolation> {
    use SignatureToken::*;

    let inner_token_opt = match token {
        Reference(token) => Some(token),
        MutableReference(token) => Some(token),
        Bool | U64 | String | ByteArray | Address | Struct(_) => None,
    };
    if let Some(inner_token) = inner_token_opt {
        if inner_token.is_reference() {
            return Some(VMStaticViolation::InvalidSignatureToken(
                token.clone(),
                token.kind(),
                inner_token.kind(),
            ));
        }
    }
    None
}
