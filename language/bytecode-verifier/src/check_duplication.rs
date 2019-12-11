// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module implements a checker for verifying that each vector in a CompiledModule contains
//! distinct values. Successful verification implies that an index in vector can be used to
//! uniquely name the entry at that index. Additionally, the checker also verifies the
//! following:
//! - struct and field definitions are consistent
//! - the handles in struct and function definitions point to IMPLEMENTED_MODULE_INDEX
//! - all struct and function handles pointing to IMPLEMENTED_MODULE_INDEX have a definition
use libra_types::vm_error::{StatusCode, VMStatus};
use std::{collections::HashSet, hash::Hash};
use vm::{
    access::ModuleAccess,
    errors::verification_error,
    file_format::{
        CompiledModule, FieldDefinitionIndex, FunctionHandleIndex, ModuleHandleIndex,
        StructFieldInformation, StructHandleIndex, TableIndex,
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

    pub fn verify(self) -> Vec<VMStatus> {
        let mut errors = vec![];

        if let Some(idx) = Self::first_duplicate_element(self.module.identifiers()) {
            errors.push(verification_error(
                IndexKind::Identifier,
                idx,
                StatusCode::DUPLICATE_ELEMENT,
            ))
        }
        if let Some(idx) = Self::first_duplicate_element(self.module.byte_array_pool()) {
            errors.push(verification_error(
                IndexKind::ByteArrayPool,
                idx,
                StatusCode::DUPLICATE_ELEMENT,
            ))
        }
        if let Some(idx) = Self::first_duplicate_element(self.module.address_pool()) {
            errors.push(verification_error(
                IndexKind::AddressPool,
                idx,
                StatusCode::DUPLICATE_ELEMENT,
            ))
        }
        if let Some(idx) = Self::first_duplicate_element(self.module.type_signatures()) {
            errors.push(verification_error(
                IndexKind::TypeSignature,
                idx,
                StatusCode::DUPLICATE_ELEMENT,
            ))
        }
        if let Some(idx) = Self::first_duplicate_element(self.module.function_signatures()) {
            errors.push(verification_error(
                IndexKind::FunctionSignature,
                idx,
                StatusCode::DUPLICATE_ELEMENT,
            ))
        }
        if let Some(idx) = Self::first_duplicate_element(self.module.locals_signatures()) {
            errors.push(verification_error(
                IndexKind::LocalsSignature,
                idx,
                StatusCode::DUPLICATE_ELEMENT,
            ))
        }
        if let Some(idx) = Self::first_duplicate_element(self.module.module_handles()) {
            errors.push(verification_error(
                IndexKind::ModuleHandle,
                idx,
                StatusCode::DUPLICATE_ELEMENT,
            ))
        }
        if let Some(idx) = Self::first_duplicate_element(
            self.module
                .struct_handles()
                .iter()
                .map(|x| (x.module, x.name)),
        ) {
            errors.push(verification_error(
                IndexKind::StructHandle,
                idx,
                StatusCode::DUPLICATE_ELEMENT,
            ))
        }
        if let Some(idx) = Self::first_duplicate_element(
            self.module
                .function_handles()
                .iter()
                .map(|x| (x.module, x.name)),
        ) {
            errors.push(verification_error(
                IndexKind::FunctionHandle,
                idx,
                StatusCode::DUPLICATE_ELEMENT,
            ))
        }
        if let Some(idx) =
            Self::first_duplicate_element(self.module.struct_defs().iter().map(|x| x.struct_handle))
        {
            errors.push(verification_error(
                IndexKind::StructDefinition,
                idx,
                StatusCode::DUPLICATE_ELEMENT,
            ))
        }
        if let Some(idx) =
            Self::first_duplicate_element(self.module.function_defs().iter().map(|x| x.function))
        {
            errors.push(verification_error(
                IndexKind::FunctionDefinition,
                idx,
                StatusCode::DUPLICATE_ELEMENT,
            ))
        }
        for (idx, function_def) in self.module.function_defs().iter().enumerate() {
            let acquires = function_def.acquires_global_resources.iter();
            if Self::first_duplicate_element(acquires).is_some() {
                errors.push(verification_error(
                    IndexKind::FunctionDefinition,
                    idx,
                    StatusCode::DUPLICATE_ACQUIRES_RESOURCE_ANNOTATION_ERROR,
                ))
            }
        }
        if let Some(idx) = Self::first_duplicate_element(
            self.module.field_defs().iter().map(|x| (x.struct_, x.name)),
        ) {
            errors.push(verification_error(
                IndexKind::FieldDefinition,
                idx,
                StatusCode::DUPLICATE_ELEMENT,
            ))
        }

        // Check that:
        // (1) the order of struct definitions matches the order of field definitions,
        // (2) each struct definition and its field definitions point to the same struct handle,
        // (3) there are no unused fields,
        // (4) each struct has at least one field. serializing a struct with zero fields is problematic
        let mut start_field_index: usize = 0;
        let mut idx_opt = None;
        for (idx, struct_def) in self.module.struct_defs().iter().enumerate() {
            let (field_count, fields) = match struct_def.field_information {
                StructFieldInformation::Native => continue,
                StructFieldInformation::Declared {
                    field_count,
                    fields,
                } => (field_count, fields),
            };
            if field_count == 0 {
                errors.push(verification_error(
                    IndexKind::StructDefinition,
                    idx,
                    StatusCode::ZERO_SIZED_STRUCT,
                ));
            }
            if FieldDefinitionIndex::new(start_field_index as u16) != fields {
                idx_opt = Some(idx);
                break;
            }
            let next_start_field_index = start_field_index + field_count as usize;
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
            errors.push(verification_error(
                IndexKind::StructDefinition,
                idx,
                StatusCode::INCONSISTENT_FIELDS,
            ));
        }

        // Check that each struct definition is pointing to module handle with index
        // IMPLEMENTED_MODULE_INDEX.
        if let Some(idx) = self.module.struct_defs().iter().position(|x| {
            self.module.struct_handle_at(x.struct_handle).module
                != ModuleHandleIndex::new(CompiledModule::IMPLEMENTED_MODULE_INDEX)
        }) {
            errors.push(verification_error(
                IndexKind::StructDefinition,
                idx,
                StatusCode::INVALID_MODULE_HANDLE,
            ))
        }
        // Check that each function definition is pointing to module handle with index
        // IMPLEMENTED_MODULE_INDEX.
        if let Some(idx) = self.module.function_defs().iter().position(|x| {
            self.module.function_handle_at(x.function).module
                != ModuleHandleIndex::new(CompiledModule::IMPLEMENTED_MODULE_INDEX)
        }) {
            errors.push(verification_error(
                IndexKind::FunctionDefinition,
                idx,
                StatusCode::INVALID_MODULE_HANDLE,
            ))
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
            errors.push(verification_error(
                IndexKind::StructHandle,
                idx,
                StatusCode::UNIMPLEMENTED_HANDLE,
            ))
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
            errors.push(verification_error(
                IndexKind::FunctionHandle,
                idx,
                StatusCode::UNIMPLEMENTED_HANDLE,
            ))
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
