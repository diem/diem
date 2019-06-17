// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module implements a checker for verifying that a non-resource struct does not
//! have resource fields inside it.
use vm::{
    errors::{VMStaticViolation, VerificationError},
    file_format::CompiledModule,
    views::ModuleView,
    IndexKind,
};

pub struct ResourceTransitiveChecker<'a> {
    module_view: ModuleView<'a, CompiledModule>,
}

impl<'a> ResourceTransitiveChecker<'a> {
    pub fn new(module: &'a CompiledModule) -> Self {
        Self {
            module_view: ModuleView::new(module),
        }
    }

    pub fn verify(self) -> Vec<VerificationError> {
        let mut errors = vec![];
        for (idx, struct_def) in self.module_view.structs().enumerate() {
            let def_is_resource = struct_def.is_resource();
            if !def_is_resource {
                let mut fields = struct_def.fields();
                let any_resource_field = fields.any(|field| field.type_signature().is_resource());
                if any_resource_field {
                    errors.push(VerificationError {
                        kind: IndexKind::StructDefinition,
                        idx,
                        err: VMStaticViolation::InvalidResourceField,
                    });
                }
            }
        }
        errors
    }
}
