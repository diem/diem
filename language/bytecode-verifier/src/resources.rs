// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module implements a checker for verifying that a non-resource struct does not
//! have resource fields inside it.
use libra_types::vm_error::{StatusCode, VMStatus};
use vm::{errors::verification_error, file_format::CompiledModule, views::ModuleView, IndexKind};

pub struct ResourceTransitiveChecker<'a> {
    module_view: ModuleView<'a, CompiledModule>,
}

impl<'a> ResourceTransitiveChecker<'a> {
    pub fn new(module: &'a CompiledModule) -> Self {
        Self {
            module_view: ModuleView::new(module),
        }
    }

    pub fn verify(self) -> Vec<VMStatus> {
        let mut errors = vec![];
        for (idx, struct_def) in self.module_view.structs().enumerate() {
            if !struct_def.is_nominal_resource() {
                let mut fields = struct_def.fields().unwrap();
                let any_resource_field = fields.any(|field| {
                    field
                        .type_signature()
                        .contains_nominal_resource(struct_def.type_formals())
                });
                if any_resource_field {
                    errors.push(verification_error(
                        IndexKind::StructDefinition,
                        idx,
                        StatusCode::INVALID_RESOURCE_FIELD,
                    ));
                }
            }
        }
        errors
    }
}
