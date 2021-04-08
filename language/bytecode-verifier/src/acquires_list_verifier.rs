// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module implements a checker for verifying properties about the acquires list on function
//! definitions. Function definitions must annotate the global resources (declared in that module)
//! accesssed by `BorrowGlobal`, `MoveFrom`, and any transitive function calls
//! The list of acquired resources (stored in `FunctionDefinition`'s `acquires_global_resources`
//! field) must have:
//! - No duplicate resources (checked by `check_duplication`)
//! - No missing resources (any resource acquired must be present)
//! - No additional resources (no extraneous resources not actually acquired)

use move_binary_format::{
    access::ModuleAccess,
    errors::{PartialVMError, PartialVMResult},
    file_format::{
        Bytecode, CodeOffset, CompiledModule, FunctionDefinition, FunctionDefinitionIndex,
        FunctionHandle, FunctionHandleIndex, StructDefinitionIndex,
    },
};
use move_core_types::vm_status::StatusCode;
use std::collections::{BTreeSet, HashMap};

pub(crate) struct AcquiresVerifier<'a> {
    module: &'a CompiledModule,
    current_function: FunctionDefinitionIndex,
    annotated_acquires: BTreeSet<StructDefinitionIndex>,
    actual_acquires: BTreeSet<StructDefinitionIndex>,
    handle_to_def: HashMap<FunctionHandleIndex, &'a FunctionDefinition>,
}

impl<'a> AcquiresVerifier<'a> {
    pub(crate) fn verify(
        module: &'a CompiledModule,
        index: FunctionDefinitionIndex,
        function_definition: &'a FunctionDefinition,
    ) -> PartialVMResult<()> {
        let annotated_acquires = function_definition
            .acquires_global_resources
            .iter()
            .cloned()
            .collect();
        let mut handle_to_def = HashMap::new();
        for func_def in module.function_defs() {
            handle_to_def.insert(func_def.function, func_def);
        }
        let mut verifier = Self {
            module,
            current_function: index,
            annotated_acquires,
            actual_acquires: BTreeSet::new(),
            handle_to_def,
        };

        for (offset, instruction) in function_definition
            .code
            .as_ref()
            .unwrap()
            .code
            .iter()
            .enumerate()
        {
            verifier.verify_instruction(instruction, offset as CodeOffset)?
        }

        for annotation in verifier.annotated_acquires {
            if !verifier.actual_acquires.contains(&annotation) {
                return Err(PartialVMError::new(
                    StatusCode::EXTRANEOUS_ACQUIRES_ANNOTATION,
                ));
            }

            let struct_def = module.struct_defs().get(annotation.0 as usize).unwrap();
            let struct_handle = module.struct_handle_at(struct_def.struct_handle);
            if !struct_handle.abilities.has_key() {
                return Err(PartialVMError::new(StatusCode::INVALID_ACQUIRES_ANNOTATION));
            }
        }

        Ok(())
    }

    fn verify_instruction(
        &mut self,
        instruction: &Bytecode,
        offset: CodeOffset,
    ) -> PartialVMResult<()> {
        match instruction {
            Bytecode::Call(idx) => self.call_acquire(*idx, offset),
            Bytecode::CallGeneric(idx) => {
                let fi = self.module.function_instantiation_at(*idx);
                self.call_acquire(fi.handle, offset)
            }
            Bytecode::MoveFrom(idx)
            | Bytecode::MutBorrowGlobal(idx)
            | Bytecode::ImmBorrowGlobal(idx) => self.struct_acquire(*idx, offset),
            Bytecode::MoveFromGeneric(idx)
            | Bytecode::MutBorrowGlobalGeneric(idx)
            | Bytecode::ImmBorrowGlobalGeneric(idx) => {
                let si = self.module.struct_instantiation_at(*idx);
                self.struct_acquire(si.def, offset)
            }
            _ => Ok(()),
        }
    }

    fn call_acquire(
        &mut self,
        fh_idx: FunctionHandleIndex,
        offset: CodeOffset,
    ) -> PartialVMResult<()> {
        let function_handle = self.module.function_handle_at(fh_idx);
        let mut function_acquired_resources =
            self.function_acquired_resources(function_handle, fh_idx);
        for acquired_resource in &function_acquired_resources {
            if !self.annotated_acquires.contains(acquired_resource) {
                return Err(self.error(StatusCode::MISSING_ACQUIRES_ANNOTATION, offset));
            }
        }
        self.actual_acquires
            .append(&mut function_acquired_resources);
        Ok(())
    }

    fn struct_acquire(
        &mut self,
        sd_idx: StructDefinitionIndex,
        offset: CodeOffset,
    ) -> PartialVMResult<()> {
        if self.annotated_acquires.contains(&sd_idx) {
            self.actual_acquires.insert(sd_idx);
            Ok(())
        } else {
            Err(self.error(StatusCode::MISSING_ACQUIRES_ANNOTATION, offset))
        }
    }

    fn function_acquired_resources(
        &self,
        function_handle: &FunctionHandle,
        fh_idx: FunctionHandleIndex,
    ) -> BTreeSet<StructDefinitionIndex> {
        if function_handle.module != self.module.self_handle_idx() {
            return BTreeSet::new();
        }
        match self.handle_to_def.get(&fh_idx) {
            Some(func_def) => func_def.acquires_global_resources.iter().cloned().collect(),
            None => BTreeSet::new(),
        }
    }

    fn error(&self, status: StatusCode, offset: CodeOffset) -> PartialVMError {
        PartialVMError::new(status).at_code_offset(self.current_function, offset)
    }
}
