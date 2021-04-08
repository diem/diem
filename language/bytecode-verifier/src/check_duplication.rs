// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module implements a checker for verifying that each vector in a CompiledModule contains
//! distinct values. Successful verification implies that an index in vector can be used to
//! uniquely name the entry at that index. Additionally, the checker also verifies the
//! following:
//! - struct and field definitions are consistent
//! - the handles in struct and function definitions point to the self module index
//! - all struct and function handles pointing to the self module index have a definition
use move_binary_format::{
    access::{ModuleAccess, ScriptAccess},
    errors::{verification_error, Location, PartialVMResult, VMResult},
    file_format::{
        CompiledModule, CompiledScript, Constant, FunctionHandle, FunctionHandleIndex,
        FunctionInstantiation, ModuleHandle, Signature, StructFieldInformation, StructHandle,
        StructHandleIndex, TableIndex,
    },
    IndexKind,
};
use move_core_types::{
    account_address::AccountAddress, identifier::Identifier, vm_status::StatusCode,
};
use std::{collections::HashSet, hash::Hash};

pub struct DuplicationChecker<'a> {
    module: &'a CompiledModule,
}

impl<'a> DuplicationChecker<'a> {
    pub fn verify_module(module: &'a CompiledModule) -> VMResult<()> {
        Self::verify_module_impl(module).map_err(|e| e.finish(Location::Module(module.self_id())))
    }

    fn verify_module_impl(module: &'a CompiledModule) -> PartialVMResult<()> {
        Self::check_identifiers(module.identifiers())?;
        Self::check_address_identifiers(module.address_identifiers())?;
        Self::check_constants(module.constant_pool())?;
        Self::check_signatures(module.signatures())?;
        Self::check_module_handles(module.module_handles())?;
        Self::check_module_handles(module.friend_decls())?;
        Self::check_struct_handles(module.struct_handles())?;
        Self::check_function_handles(module.function_handles())?;
        Self::check_function_instantiations(module.function_instantiations())?;

        let checker = Self { module };
        checker.check_field_handles()?;
        checker.check_field_instantiations()?;
        checker.check_function_defintions()?;
        checker.check_struct_definitions()?;
        checker.check_struct_instantiations()
    }

    pub fn verify_script(module: &'a CompiledScript) -> VMResult<()> {
        Self::verify_script_impl(module).map_err(|e| e.finish(Location::Script))
    }

    fn verify_script_impl(script: &'a CompiledScript) -> PartialVMResult<()> {
        Self::check_identifiers(script.identifiers())?;
        Self::check_address_identifiers(script.address_identifiers())?;
        Self::check_constants(script.constant_pool())?;
        Self::check_signatures(script.signatures())?;
        Self::check_module_handles(script.module_handles())?;
        Self::check_struct_handles(script.struct_handles())?;
        Self::check_function_handles(script.function_handles())?;
        Self::check_function_instantiations(script.function_instantiations())
    }

    fn check_identifiers(identifiers: &[Identifier]) -> PartialVMResult<()> {
        match Self::first_duplicate_element(identifiers) {
            Some(idx) => Err(verification_error(
                StatusCode::DUPLICATE_ELEMENT,
                IndexKind::Identifier,
                idx,
            )),
            None => Ok(()),
        }
    }

    fn check_address_identifiers(address_identifiers: &[AccountAddress]) -> PartialVMResult<()> {
        match Self::first_duplicate_element(address_identifiers) {
            Some(idx) => Err(verification_error(
                StatusCode::DUPLICATE_ELEMENT,
                IndexKind::AddressIdentifier,
                idx,
            )),
            None => Ok(()),
        }
    }

    fn check_constants(constant_pool: &[Constant]) -> PartialVMResult<()> {
        match Self::first_duplicate_element(constant_pool) {
            Some(idx) => Err(verification_error(
                StatusCode::DUPLICATE_ELEMENT,
                IndexKind::ConstantPool,
                idx,
            )),
            None => Ok(()),
        }
    }

    fn check_signatures(signatures: &[Signature]) -> PartialVMResult<()> {
        match Self::first_duplicate_element(signatures) {
            Some(idx) => Err(verification_error(
                StatusCode::DUPLICATE_ELEMENT,
                IndexKind::Signature,
                idx,
            )),
            None => Ok(()),
        }
    }

    fn check_module_handles(module_handles: &[ModuleHandle]) -> PartialVMResult<()> {
        match Self::first_duplicate_element(module_handles) {
            Some(idx) => Err(verification_error(
                StatusCode::DUPLICATE_ELEMENT,
                IndexKind::ModuleHandle,
                idx,
            )),
            None => Ok(()),
        }
    }

    // StructHandles - module and name define uniqueness
    fn check_struct_handles(struct_handles: &[StructHandle]) -> PartialVMResult<()> {
        match Self::first_duplicate_element(struct_handles.iter().map(|x| (x.module, x.name))) {
            Some(idx) => Err(verification_error(
                StatusCode::DUPLICATE_ELEMENT,
                IndexKind::StructHandle,
                idx,
            )),
            None => Ok(()),
        }
    }

    fn check_function_instantiations(
        function_instantiations: &[FunctionInstantiation],
    ) -> PartialVMResult<()> {
        match Self::first_duplicate_element(function_instantiations) {
            Some(idx) => Err(verification_error(
                StatusCode::DUPLICATE_ELEMENT,
                IndexKind::FunctionInstantiation,
                idx,
            )),
            None => Ok(()),
        }
    }

    // FunctionHandles - module and name define uniqueness
    fn check_function_handles(function_handles: &[FunctionHandle]) -> PartialVMResult<()> {
        match Self::first_duplicate_element(function_handles.iter().map(|x| (x.module, x.name))) {
            Some(idx) => Err(verification_error(
                StatusCode::DUPLICATE_ELEMENT,
                IndexKind::FunctionHandle,
                idx,
            )),
            None => Ok(()),
        }
    }

    //
    // Module only code
    //

    fn check_field_handles(&self) -> PartialVMResult<()> {
        match Self::first_duplicate_element(self.module.field_handles()) {
            Some(idx) => Err(verification_error(
                StatusCode::DUPLICATE_ELEMENT,
                IndexKind::FieldHandle,
                idx,
            )),
            None => Ok(()),
        }
    }

    fn check_struct_instantiations(&self) -> PartialVMResult<()> {
        match Self::first_duplicate_element(self.module.struct_instantiations()) {
            Some(idx) => Err(verification_error(
                StatusCode::DUPLICATE_ELEMENT,
                IndexKind::StructDefInstantiation,
                idx,
            )),
            None => Ok(()),
        }
    }

    fn check_field_instantiations(&self) -> PartialVMResult<()> {
        if let Some(idx) = Self::first_duplicate_element(self.module.field_instantiations()) {
            return Err(verification_error(
                StatusCode::DUPLICATE_ELEMENT,
                IndexKind::FieldInstantiation,
                idx,
            ));
        }
        Ok(())
    }

    fn check_struct_definitions(&self) -> PartialVMResult<()> {
        // StructDefinition - contained StructHandle defines uniqueness
        if let Some(idx) =
            Self::first_duplicate_element(self.module.struct_defs().iter().map(|x| x.struct_handle))
        {
            return Err(verification_error(
                StatusCode::DUPLICATE_ELEMENT,
                IndexKind::StructDefinition,
                idx,
            ));
        }
        // Field names in structs must be unique
        for (struct_idx, struct_def) in self.module.struct_defs().iter().enumerate() {
            let fields = match &struct_def.field_information {
                StructFieldInformation::Native => continue,
                StructFieldInformation::Declared(fields) => fields,
            };
            if fields.is_empty() {
                return Err(verification_error(
                    StatusCode::ZERO_SIZED_STRUCT,
                    IndexKind::StructDefinition,
                    struct_idx as TableIndex,
                ));
            }
            if let Some(idx) = Self::first_duplicate_element(fields.iter().map(|x| x.name)) {
                return Err(verification_error(
                    StatusCode::DUPLICATE_ELEMENT,
                    IndexKind::FieldDefinition,
                    idx,
                ));
            }
        }
        // Check that each struct definition is pointing to the self module
        if let Some(idx) = self.module.struct_defs().iter().position(|x| {
            self.module.struct_handle_at(x.struct_handle).module != self.module.self_handle_idx()
        }) {
            return Err(verification_error(
                StatusCode::INVALID_MODULE_HANDLE,
                IndexKind::StructDefinition,
                idx as TableIndex,
            ));
        }
        // Check that each struct handle in self module is implemented (has a declaration)
        let implemented_struct_handles: HashSet<StructHandleIndex> = self
            .module
            .struct_defs()
            .iter()
            .map(|x| x.struct_handle)
            .collect();
        if let Some(idx) = (0..self.module.struct_handles().len()).position(|x| {
            let y = StructHandleIndex::new(x as u16);
            self.module.struct_handle_at(y).module == self.module.self_handle_idx()
                && !implemented_struct_handles.contains(&y)
        }) {
            return Err(verification_error(
                StatusCode::UNIMPLEMENTED_HANDLE,
                IndexKind::StructHandle,
                idx as TableIndex,
            ));
        }
        Ok(())
    }

    fn check_function_defintions(&self) -> PartialVMResult<()> {
        // FunctionDefinition - contained FunctionHandle defines uniqueness
        if let Some(idx) =
            Self::first_duplicate_element(self.module.function_defs().iter().map(|x| x.function))
        {
            return Err(verification_error(
                StatusCode::DUPLICATE_ELEMENT,
                IndexKind::FunctionDefinition,
                idx,
            ));
        }
        // Acquires in function declarations contain unique struct definitions
        for (idx, function_def) in self.module.function_defs().iter().enumerate() {
            let acquires = function_def.acquires_global_resources.iter();
            if Self::first_duplicate_element(acquires).is_some() {
                return Err(verification_error(
                    StatusCode::DUPLICATE_ACQUIRES_ANNOTATION,
                    IndexKind::FunctionDefinition,
                    idx as TableIndex,
                ));
            }
        }
        // Check that each function definition is pointing to the self module
        if let Some(idx) = self.module.function_defs().iter().position(|x| {
            self.module.function_handle_at(x.function).module != self.module.self_handle_idx()
        }) {
            return Err(verification_error(
                StatusCode::INVALID_MODULE_HANDLE,
                IndexKind::FunctionDefinition,
                idx as TableIndex,
            ));
        }
        // Check that each function handle in self module is implemented (has a declaration)
        let implemented_function_handles: HashSet<FunctionHandleIndex> = self
            .module
            .function_defs()
            .iter()
            .map(|x| x.function)
            .collect();
        if let Some(idx) = (0..self.module.function_handles().len()).position(|x| {
            let y = FunctionHandleIndex::new(x as u16);
            self.module.function_handle_at(y).module == self.module.self_handle_idx()
                && !implemented_function_handles.contains(&y)
        }) {
            return Err(verification_error(
                StatusCode::UNIMPLEMENTED_HANDLE,
                IndexKind::FunctionHandle,
                idx as TableIndex,
            ));
        }
        Ok(())
    }

    fn first_duplicate_element<T>(iter: T) -> Option<TableIndex>
    where
        T: IntoIterator,
        T::Item: Eq + Hash,
    {
        let mut uniq = HashSet::new();
        for (i, x) in iter.into_iter().enumerate() {
            if !uniq.insert(x) {
                return Some(i as TableIndex);
            }
        }
        None
    }
}
