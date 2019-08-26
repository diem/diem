// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module implements a checker for verifying signature tokens used in types of function
//! parameters, locals, and fields of structs are well-formed. References can only occur at the
//! top-level in all tokens.  Additionally, references cannot occur at all in field types.
use types::vm_error::{StatusCode, VMStatus};
use vm::{
    access::ModuleAccess,
    errors::append_err_info,
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

    pub fn verify(self) -> Vec<VMStatus> {
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
                check_signature_refs(&view)
                    .map(move |err| append_err_info(err, IndexKind::FieldDefinition, idx))
            })
            .collect();
        errors.push(signature_ref_errors);

        errors.into_iter().flatten().collect()
    }

    #[inline]
    fn verify_impl(
        kind: IndexKind,
        views: impl Iterator<Item = impl SignatureCheck>,
    ) -> Vec<VMStatus> {
        views
            .enumerate()
            .map(move |(idx, view)| {
                view.check_signatures()
                    .into_iter()
                    .map(move |err| append_err_info(err, kind, idx))
            })
            .flatten()
            .collect()
    }
}

trait SignatureCheck {
    fn check_signatures(&self) -> Vec<VMStatus>;
}

impl<'a, T: ModuleAccess> SignatureCheck for FunctionSignatureView<'a, T> {
    fn check_signatures(&self) -> Vec<VMStatus> {
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
    fn check_signatures(&self) -> Vec<VMStatus> {
        check_structure(self.token().as_inner())
            .into_iter()
            .collect()
    }
}

impl<'a, T: ModuleAccess> SignatureCheck for LocalsSignatureView<'a, T> {
    fn check_signatures(&self) -> Vec<VMStatus> {
        self.tokens()
            .filter_map(|token| check_structure(token.as_inner()))
            .collect()
    }
}

/// Field definitions have additional constraints on signatures -- field signatures cannot be
/// references or mutable references.
pub(crate) fn check_signature_refs(
    view: &FieldDefinitionView<'_, CompiledModule>,
) -> Option<VMStatus> {
    let type_signature = view.type_signature();
    let token = type_signature.token();
    let kind = token.signature_token_kind();
    match kind {
        SignatureTokenKind::Reference | SignatureTokenKind::MutableReference => {
            Some(VMStatus::new(StatusCode::INVALID_FIELD_DEF_REFERENCE))
        }
        SignatureTokenKind::Value => None,
    }
}

/// Check that this token is structurally correct.
/// In particular, check that the token has a reference only at the top level.
pub(crate) fn check_structure(token: &SignatureToken) -> Option<VMStatus> {
    use SignatureToken::*;

    let inner_token_opt = match token {
        Reference(token) => Some(token),
        MutableReference(token) => Some(token),
        Bool | U64 | String | ByteArray | Address | Struct(_, _) | TypeParameter(_) => None,
    };
    if let Some(inner_token) = inner_token_opt {
        if inner_token.is_reference() {
            let msg = format!(
                "Invalid token {:#?} of kind {} with inner token {}",
                token.clone(),
                token.signature_token_kind(),
                inner_token.signature_token_kind(),
            );
            return Some(VMStatus::new(StatusCode::INVALID_SIGNATURE_TOKEN).with_message(msg));
        }
    }
    None
}
