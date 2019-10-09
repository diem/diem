// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module implements a checker for verifying properties about the acquires list on function
//! definitions. Function definitions must annotate the global resources (declared in that module)
//! accesssed by `BorrowGlobal`, `MoveFrom`, and any transitive function calls
//! The list of acquired resources (stored in `FunctionDefinition`'s `acquires_global_resources`
//! field) must have:
//! - No duplicate resources (checked by `check_duplication`)
//! - No missing resources (any resource acquired must be present)
//! - No additional resources (no extraneous resources not actually acquired)

use libra_types::vm_error::{StatusCode, VMStatus};
use std::collections::BTreeSet;
use vm::{
    access::ModuleAccess,
    errors::err_at_offset,
    file_format::{Bytecode, CompiledModule, FunctionDefinition, StructDefinitionIndex},
    views::{FunctionDefinitionView, ModuleView, StructDefinitionView, ViewInternals},
};

pub struct AcquiresVerifier<'a> {
    module_view: ModuleView<'a, CompiledModule>,
    annotated_acquires: BTreeSet<StructDefinitionIndex>,
    actual_acquires: BTreeSet<StructDefinitionIndex>,
    errors: Vec<VMStatus>,
}

impl<'a> AcquiresVerifier<'a> {
    pub fn verify(
        module: &'a CompiledModule,
        function_definition: &'a FunctionDefinition,
    ) -> Vec<VMStatus> {
        let annotated_acquires = function_definition
            .acquires_global_resources
            .iter()
            .cloned()
            .collect();
        let mut verifier = Self {
            module_view: ModuleView::new(module),
            annotated_acquires,
            actual_acquires: BTreeSet::new(),
            errors: vec![],
        };

        let function_definition_view = FunctionDefinitionView::new(module, function_definition);
        for (offset, instruction) in function_definition_view.code().code.iter().enumerate() {
            verifier.verify_instruction(instruction, offset)
        }

        for annotation in verifier.annotated_acquires {
            if !verifier.actual_acquires.contains(&annotation) {
                verifier.errors.push(VMStatus::new(
                    StatusCode::EXTRANEOUS_ACQUIRES_RESOURCE_ANNOTATION_ERROR,
                ))
            }

            let struct_def = module.struct_defs().get(annotation.0 as usize).unwrap();
            let struct_def_view = StructDefinitionView::new(module, struct_def);
            if !struct_def_view.is_nominal_resource() {
                verifier.errors.push(VMStatus::new(
                    StatusCode::INVALID_ACQUIRES_RESOURCE_ANNOTATION_ERROR,
                ))
            }
        }

        verifier.errors
    }

    fn verify_instruction(&mut self, instruction: &Bytecode, offset: usize) {
        match instruction {
            Bytecode::Call(idx, _) => {
                let function_handle = self.module_view.as_inner().function_handle_at(*idx);
                let mut function_acquired_resources = self
                    .module_view
                    .function_acquired_resources(&function_handle);
                for acquired_resource in &function_acquired_resources {
                    if !self.annotated_acquires.contains(acquired_resource) {
                        self.errors.push(err_at_offset(
                            StatusCode::MISSING_ACQUIRES_RESOURCE_ANNOTATION_ERROR,
                            offset,
                        ))
                    }
                }
                self.actual_acquires
                    .append(&mut function_acquired_resources)
            }
            Bytecode::MoveFrom(idx, _)
            | Bytecode::MutBorrowGlobal(idx, _)
            | Bytecode::ImmBorrowGlobal(idx, _) => {
                if !self.annotated_acquires.contains(idx) {
                    self.errors.push(err_at_offset(
                        StatusCode::MISSING_ACQUIRES_RESOURCE_ANNOTATION_ERROR,
                        offset,
                    ))
                }
                self.actual_acquires.insert(*idx);
            }
            _ => (),
        }
    }
}
