// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module implements a checker for verifying signature tokens used in types of function
//! parameters, locals, and fields of structs are well-formed. References can only occur at the
//! top-level in all tokens.  Additionally, references cannot occur at all in field types.
use vm::{
    checks::SignatureCheck, errors::VerificationError, file_format::CompiledModule,
    views::ModuleView, IndexKind,
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
                view.check_signature_refs()
                    .map(move |err| VerificationError {
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
