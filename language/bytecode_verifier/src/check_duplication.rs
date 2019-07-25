// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module implements a checker for verifying that each vector in a CompiledModule contains
//! distinct values. Successful verification implies that an index in vector can be used to
//! uniquely name the entry at that index. Additionally, the checker also verifies the
//! following:
//! - struct and field definitions are consistent
//! - the handles in struct and function definitions point to IMPLEMENTED_MODULE_INDEX
//! - all struct and function handles pointing to IMPLEMENTED_MODULE_INDEX have a definition
use std::{collections::HashSet, hash::Hash};
use vm::{
    access::ModuleAccess,
    errors::{VMStaticViolation, VerificationError},
    file_format::{
        CompiledModule, FieldDefinitionIndex, FunctionHandleIndex, ModuleHandleIndex,
        StructHandleIndex, TableIndex,
    },
    IndexKind,
};

pub struct DuplicationChecker<'a> {
    module: &'a CompiledModule,
}

impl<'a> DuplicationChecker<'a> {
    pub fn new(module: &'a CompiledModule) -> Self {
        Self { module }
    }

    pub fn verify(self) -> Vec<VerificationError> {
        let mut errors = vec![];

        if let Some(idx) = Self::first_duplicate_element(self.module.string_pool()) {
            errors.push(VerificationError {
                kind: IndexKind::StringPool,
                idx,
                err: VMStaticViolation::DuplicateElement,
            })
        }
        if let Some(idx) = Self::first_duplicate_element(self.module.byte_array_pool()) {
            errors.push(VerificationError {
                kind: IndexKind::ByteArrayPool,
                idx,
                err: VMStaticViolation::DuplicateElement,
            })
        }
        if let Some(idx) = Self::first_duplicate_element(self.module.address_pool()) {
            errors.push(VerificationError {
                kind: IndexKind::AddressPool,
                idx,
                err: VMStaticViolation::DuplicateElement,
            })
        }
        if let Some(idx) = Self::first_duplicate_element(self.module.type_signatures()) {
            errors.push(VerificationError {
                kind: IndexKind::TypeSignature,
                idx,
                err: VMStaticViolation::DuplicateElement,
            })
        }
        if let Some(idx) = Self::first_duplicate_element(self.module.function_signatures()) {
            errors.push(VerificationError {
                kind: IndexKind::FunctionSignature,
                idx,
                err: VMStaticViolation::DuplicateElement,
            })
        }
        if let Some(idx) = Self::first_duplicate_element(self.module.locals_signatures()) {
            errors.push(VerificationError {
                kind: IndexKind::LocalsSignature,
                idx,
                err: VMStaticViolation::DuplicateElement,
            })
        }
        if let Some(idx) = Self::first_duplicate_element(self.module.module_handles()) {
            errors.push(VerificationError {
                kind: IndexKind::ModuleHandle,
                idx,
                err: VMStaticViolation::DuplicateElement,
            })
        }
        if let Some(idx) = Self::first_duplicate_element(
            self.module
                .struct_handles()
                .iter()
                .map(|x| (x.module, x.name)),
        ) {
            errors.push(VerificationError {
                kind: IndexKind::StructHandle,
                idx,
                err: VMStaticViolation::DuplicateElement,
            })
        }
        if let Some(idx) = Self::first_duplicate_element(
            self.module
                .function_handles()
                .iter()
                .map(|x| (x.module, x.name)),
        ) {
            errors.push(VerificationError {
                kind: IndexKind::FunctionHandle,
                idx,
                err: VMStaticViolation::DuplicateElement,
            })
        }
        if let Some(idx) =
            Self::first_duplicate_element(self.module.struct_defs().iter().map(|x| x.struct_handle))
        {
            errors.push(VerificationError {
                kind: IndexKind::StructDefinition,
                idx,
                err: VMStaticViolation::DuplicateElement,
            })
        }
        if let Some(idx) =
            Self::first_duplicate_element(self.module.function_defs().iter().map(|x| x.function))
        {
            errors.push(VerificationError {
                kind: IndexKind::FunctionDefinition,
                idx,
                err: VMStaticViolation::DuplicateElement,
            })
        }
        if let Some(idx) = Self::first_duplicate_element(
            self.module.field_defs().iter().map(|x| (x.struct_, x.name)),
        ) {
            errors.push(VerificationError {
                kind: IndexKind::FieldDefinition,
                idx,
                err: VMStaticViolation::DuplicateElement,
            })
        }

        // Check that:
        // (1) the order of struct definitions matches the order of field definitions,
        // (2) each struct definition and its field definitions point to the same struct handle,
        // (3) there are no unused fields.
        let mut start_field_index: usize = 0;
        let mut idx_opt = None;
        for (idx, struct_def) in self.module.struct_defs().iter().enumerate() {
            if FieldDefinitionIndex::new(start_field_index as u16) != struct_def.fields {
                idx_opt = Some(idx);
                break;
            }
            let next_start_field_index = start_field_index + struct_def.field_count as usize;
            let all_fields_match = (start_field_index..next_start_field_index).all(|i| {
                struct_def.struct_handle
                    == self
                        .module
                        .field_def_at(FieldDefinitionIndex::new(i as TableIndex))
                        .struct_
            });
            if !all_fields_match {
                idx_opt = Some(idx);
                break;
            }
            start_field_index = next_start_field_index;
        }
        if let Some(idx) = idx_opt {
            errors.push(VerificationError {
                kind: IndexKind::StructDefinition,
                idx,
                err: VMStaticViolation::InconsistentFields,
            });
        } else if start_field_index != self.module.field_defs().len() {
            errors.push(VerificationError {
                kind: IndexKind::FieldDefinition,
                idx: start_field_index,
                err: VMStaticViolation::UnusedFields,
            });
        }

        // Check that each struct definition is pointing to module handle with index
        // IMPLEMENTED_MODULE_INDEX.
        if let Some(idx) = self.module.struct_defs().iter().position(|x| {
            self.module.struct_handle_at(x.struct_handle).module
                != ModuleHandleIndex::new(CompiledModule::IMPLEMENTED_MODULE_INDEX)
        }) {
            errors.push(VerificationError {
                kind: IndexKind::StructDefinition,
                idx,
                err: VMStaticViolation::InvalidModuleHandle,
            })
        }
        // Check that each function definition is pointing to module handle with index
        // IMPLEMENTED_MODULE_INDEX.
        if let Some(idx) = self.module.function_defs().iter().position(|x| {
            self.module.function_handle_at(x.function).module
                != ModuleHandleIndex::new(CompiledModule::IMPLEMENTED_MODULE_INDEX)
        }) {
            errors.push(VerificationError {
                kind: IndexKind::FunctionDefinition,
                idx,
                err: VMStaticViolation::InvalidModuleHandle,
            })
        }
        // Check that each struct handle with module handle index IMPLEMENTED_MODULE_INDEX is
        // implemented.
        let implemented_struct_handles: HashSet<StructHandleIndex> = self
            .module
            .struct_defs()
            .iter()
            .map(|x| x.struct_handle)
            .collect();
        if let Some(idx) = (0..self.module.struct_handles().len()).position(|x| {
            let y = StructHandleIndex::new(x as u16);
            self.module.struct_handle_at(y).module
                == ModuleHandleIndex::new(CompiledModule::IMPLEMENTED_MODULE_INDEX)
                && !implemented_struct_handles.contains(&y)
        }) {
            errors.push(VerificationError {
                kind: IndexKind::StructHandle,
                idx,
                err: VMStaticViolation::UnimplementedHandle,
            })
        }
        // Check that each function handle with module handle index IMPLEMENTED_MODULE_INDEX is
        // implemented.
        let implemented_function_handles: HashSet<FunctionHandleIndex> = self
            .module
            .function_defs()
            .iter()
            .map(|x| x.function)
            .collect();
        if let Some(idx) = (0..self.module.function_handles().len()).position(|x| {
            let y = FunctionHandleIndex::new(x as u16);
            self.module.function_handle_at(y).module
                == ModuleHandleIndex::new(CompiledModule::IMPLEMENTED_MODULE_INDEX)
                && !implemented_function_handles.contains(&y)
        }) {
            errors.push(VerificationError {
                kind: IndexKind::FunctionHandle,
                idx,
                err: VMStaticViolation::UnimplementedHandle,
            })
        }

        errors
    }

    fn first_duplicate_element<T>(iter: T) -> Option<usize>
    where
        T: IntoIterator,
        T::Item: Eq + Hash,
    {
        let mut uniq = HashSet::new();
        for (i, x) in iter.into_iter().enumerate() {
            if !uniq.insert(x) {
                return Some(i);
            }
        }
        None
    }
}
