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
    errors::{err_at_offset, VMResult},
    file_format::{
        Bytecode, CompiledModule, FunctionDefinition, FunctionHandleIndex, StructDefinitionIndex,
    },
    views::{FunctionDefinitionView, ModuleView, StructDefinitionView, ViewInternals},
};

pub struct AcquiresVerifier<'a> {
    module_view: ModuleView<'a, CompiledModule>,
    annotated_acquires: BTreeSet<StructDefinitionIndex>,
    actual_acquires: BTreeSet<StructDefinitionIndex>,
}

impl<'a> AcquiresVerifier<'a> {
    pub fn verify(
        module: &'a CompiledModule,
        function_definition: &'a FunctionDefinition,
    ) -> VMResult<()> {
        let annotated_acquires = function_definition
            .acquires_global_resources
            .iter()
            .cloned()
            .collect();

        let mut verifier = Self {
            module_view: ModuleView::new(module),
            annotated_acquires,
            actual_acquires: BTreeSet::new(),
        };

        let function_definition_view = FunctionDefinitionView::new(module, function_definition);
        for (offset, instruction) in function_definition_view
            .code()
            .unwrap()
            .code
            .iter()
            .enumerate()
        {
            verifier.verify_instruction(instruction, offset)?
        }

        for annotation in verifier.annotated_acquires {
            if !verifier.actual_acquires.contains(&annotation) {
                return Err(VMStatus::new(
                    StatusCode::EXTRANEOUS_ACQUIRES_RESOURCE_ANNOTATION_ERROR,
                ));
            }

            let struct_def = module.struct_defs().get(annotation.0 as usize).unwrap();
            let struct_def_view = StructDefinitionView::new(module, struct_def);
            if !struct_def_view.is_nominal_resource() {
                return Err(VMStatus::new(
                    StatusCode::INVALID_ACQUIRES_RESOURCE_ANNOTATION_ERROR,
                ));
            }
        }

        Ok(())
    }

    fn verify_instruction(&mut self, instruction: &Bytecode, offset: usize) -> VMResult<()> {
        match instruction {
            Bytecode::Call(idx) => self.call_acquire(*idx, offset),
            Bytecode::CallGeneric(idx) => {
                let fi = self.module_view.as_inner().function_instantiation_at(*idx);
                self.call_acquire(fi.handle, offset)
            }
            Bytecode::MoveFrom(idx)
            | Bytecode::MutBorrowGlobal(idx)
            | Bytecode::ImmBorrowGlobal(idx) => self.struct_acquire(*idx, offset),
            Bytecode::MoveFromGeneric(idx)
            | Bytecode::MutBorrowGlobalGeneric(idx)
            | Bytecode::ImmBorrowGlobalGeneric(idx) => {
                let si = self.module_view.as_inner().struct_instantiation_at(*idx);
                self.struct_acquire(si.def, offset)
            }
            _ => Ok(()),
        }
    }

    fn call_acquire(&mut self, fh_idx: FunctionHandleIndex, offset: usize) -> VMResult<()> {
        let function_handle = self.module_view.as_inner().function_handle_at(fh_idx);
        let mut function_acquired_resources = self
            .module_view
            .function_acquired_resources(function_handle);
        for acquired_resource in &function_acquired_resources {
            if !self.annotated_acquires.contains(acquired_resource) {
                return Err(err_at_offset(
                    StatusCode::MISSING_ACQUIRES_RESOURCE_ANNOTATION_ERROR,
                    offset,
                ));
            }
        }
        self.actual_acquires
            .append(&mut function_acquired_resources);
        Ok(())
    }

    fn struct_acquire(&mut self, sd_idx: StructDefinitionIndex, offset: usize) -> VMResult<()> {
        if self.annotated_acquires.contains(&sd_idx) {
            self.actual_acquires.insert(sd_idx);
            Ok(())
        } else {
            Err(err_at_offset(
                StatusCode::MISSING_ACQUIRES_RESOURCE_ANNOTATION_ERROR,
                offset,
            ))
        }
    }
}
