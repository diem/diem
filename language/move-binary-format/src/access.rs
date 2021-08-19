// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Defines accessors for compiled modules.

use crate::{file_format::*, internals::ModuleIndex};
use move_core_types::{
    account_address::AccountAddress,
    identifier::{IdentStr, Identifier},
    language_storage::ModuleId,
};

/// Represents accessors for a compiled module.
///
/// This is a trait to allow working across different wrappers for `CompiledModule`.
pub trait ModuleAccess: Sync {
    /// Returns the `CompiledModule` that will be used for accesses.
    fn as_module(&self) -> &CompiledModule;

    fn self_handle_idx(&self) -> ModuleHandleIndex {
        self.as_module().self_module_handle_idx
    }

    /// Returns the `ModuleHandle` for `self`.
    fn self_handle(&self) -> &ModuleHandle {
        assume_preconditions!(); // invariant
        let handle = self.module_handle_at(self.self_handle_idx());
        assumed_postcondition!(
            handle.address.into_index() < self.as_module().address_identifiers.len()
        ); // invariant
        assumed_postcondition!(handle.name.into_index() < self.as_module().identifiers.len()); // invariant
        handle
    }

    /// Returns the name of the module.
    fn name(&self) -> &IdentStr {
        self.identifier_at(self.self_handle().name)
    }

    /// Returns the address of the module.
    fn address(&self) -> &AccountAddress {
        self.address_identifier_at(self.self_handle().address)
    }

    fn module_handle_at(&self, idx: ModuleHandleIndex) -> &ModuleHandle {
        let handle = &self.as_module().module_handles[idx.into_index()];
        assumed_postcondition!(
            handle.address.into_index() < self.as_module().address_identifiers.len()
        ); // invariant
        assumed_postcondition!(handle.name.into_index() < self.as_module().identifiers.len()); // invariant
        handle
    }

    fn struct_handle_at(&self, idx: StructHandleIndex) -> &StructHandle {
        let handle = &self.as_module().struct_handles[idx.into_index()];
        assumed_postcondition!(handle.module.into_index() < self.as_module().module_handles.len()); // invariant
        handle
    }

    fn function_handle_at(&self, idx: FunctionHandleIndex) -> &FunctionHandle {
        let handle = &self.as_module().function_handles[idx.into_index()];
        assumed_postcondition!(handle.parameters.into_index() < self.as_module().signatures.len()); // invariant
        assumed_postcondition!(handle.return_.into_index() < self.as_module().signatures.len()); // invariant
        handle
    }

    fn field_handle_at(&self, idx: FieldHandleIndex) -> &FieldHandle {
        let handle = &self.as_module().field_handles[idx.into_index()];
        assumed_postcondition!(handle.owner.into_index() < self.as_module().struct_defs.len()); // invariant
        handle
    }

    fn struct_instantiation_at(&self, idx: StructDefInstantiationIndex) -> &StructDefInstantiation {
        &self.as_module().struct_def_instantiations[idx.into_index()]
    }

    fn function_instantiation_at(&self, idx: FunctionInstantiationIndex) -> &FunctionInstantiation {
        &self.as_module().function_instantiations[idx.into_index()]
    }

    fn field_instantiation_at(&self, idx: FieldInstantiationIndex) -> &FieldInstantiation {
        &self.as_module().field_instantiations[idx.into_index()]
    }

    fn signature_at(&self, idx: SignatureIndex) -> &Signature {
        &self.as_module().signatures[idx.into_index()]
    }

    fn identifier_at(&self, idx: IdentifierIndex) -> &IdentStr {
        &self.as_module().identifiers[idx.into_index()]
    }

    fn address_identifier_at(&self, idx: AddressIdentifierIndex) -> &AccountAddress {
        &self.as_module().address_identifiers[idx.into_index()]
    }

    fn constant_at(&self, idx: ConstantPoolIndex) -> &Constant {
        &self.as_module().constant_pool[idx.into_index()]
    }

    fn struct_def_at(&self, idx: StructDefinitionIndex) -> &StructDefinition {
        &self.as_module().struct_defs[idx.into_index()]
    }

    fn function_def_at(&self, idx: FunctionDefinitionIndex) -> &FunctionDefinition {
        let result = &self.as_module().function_defs[idx.into_index()];
        assumed_postcondition!(result.function.into_index() < self.function_handles().len()); // invariant
        assumed_postcondition!(match &result.code {
            Some(code) => code.locals.into_index() < self.signatures().len(),
            None => true,
        }); // invariant
        result
    }

    fn module_handles(&self) -> &[ModuleHandle] {
        &self.as_module().module_handles
    }

    fn struct_handles(&self) -> &[StructHandle] {
        &self.as_module().struct_handles
    }

    fn function_handles(&self) -> &[FunctionHandle] {
        &self.as_module().function_handles
    }

    fn field_handles(&self) -> &[FieldHandle] {
        &self.as_module().field_handles
    }

    fn struct_instantiations(&self) -> &[StructDefInstantiation] {
        &self.as_module().struct_def_instantiations
    }

    fn function_instantiations(&self) -> &[FunctionInstantiation] {
        &self.as_module().function_instantiations
    }

    fn field_instantiations(&self) -> &[FieldInstantiation] {
        &self.as_module().field_instantiations
    }

    fn signatures(&self) -> &[Signature] {
        &self.as_module().signatures
    }

    fn constant_pool(&self) -> &[Constant] {
        &self.as_module().constant_pool
    }

    fn identifiers(&self) -> &[Identifier] {
        &self.as_module().identifiers
    }

    fn address_identifiers(&self) -> &[AccountAddress] {
        &self.as_module().address_identifiers
    }

    fn struct_defs(&self) -> &[StructDefinition] {
        &self.as_module().struct_defs
    }

    fn function_defs(&self) -> &[FunctionDefinition] {
        &self.as_module().function_defs
    }

    fn friend_decls(&self) -> &[ModuleHandle] {
        &self.as_module().friend_decls
    }

    fn module_id_for_handle(&self, module_handle_idx: &ModuleHandle) -> ModuleId {
        self.as_module().module_id_for_handle(module_handle_idx)
    }

    fn self_id(&self) -> ModuleId {
        self.as_module().self_id()
    }

    fn version(&self) -> u32 {
        self.as_module().version
    }

    fn immediate_dependencies(&self) -> Vec<ModuleId> {
        let self_handle = self.self_handle();
        self.module_handles()
            .iter()
            .filter(|&handle| handle != self_handle)
            .map(|handle| self.module_id_for_handle(handle))
            .collect()
    }

    fn immediate_friends(&self) -> Vec<ModuleId> {
        self.friend_decls()
            .iter()
            .map(|handle| self.module_id_for_handle(handle))
            .collect()
    }

    fn find_struct_def(&self, idx: StructHandleIndex) -> Option<&StructDefinition> {
        self.struct_defs().iter().find(|d| d.struct_handle == idx)
    }

    fn find_struct_def_by_name(&self, name: &IdentStr) -> Option<&StructDefinition> {
        self.struct_defs().iter().find(|def| {
            let handle = self.struct_handle_at(def.struct_handle);
            name == self.identifier_at(handle.name)
        })
    }
}

/// Represents accessors for a compiled script.
///
/// This is a trait to allow working across different wrappers for `CompiledScript`.
pub trait ScriptAccess: Sync {
    /// Returns the `CompiledScript` that will be used for accesses.
    fn as_script(&self) -> &CompiledScript;

    fn module_handle_at(&self, idx: ModuleHandleIndex) -> &ModuleHandle {
        &self.as_script().module_handles[idx.into_index()]
    }

    fn struct_handle_at(&self, idx: StructHandleIndex) -> &StructHandle {
        &self.as_script().struct_handles[idx.into_index()]
    }

    fn function_handle_at(&self, idx: FunctionHandleIndex) -> &FunctionHandle {
        &self.as_script().function_handles[idx.into_index()]
    }

    fn signature_at(&self, idx: SignatureIndex) -> &Signature {
        &self.as_script().signatures[idx.into_index()]
    }

    fn identifier_at(&self, idx: IdentifierIndex) -> &IdentStr {
        &self.as_script().identifiers[idx.into_index()]
    }

    fn address_identifier_at(&self, idx: AddressIdentifierIndex) -> &AccountAddress {
        &self.as_script().address_identifiers[idx.into_index()]
    }

    fn constant_at(&self, idx: ConstantPoolIndex) -> &Constant {
        &self.as_script().constant_pool[idx.into_index()]
    }

    fn function_instantiation_at(&self, idx: FunctionInstantiationIndex) -> &FunctionInstantiation {
        &self.as_script().function_instantiations[idx.into_index()]
    }

    fn module_handles(&self) -> &[ModuleHandle] {
        &self.as_script().module_handles
    }

    fn struct_handles(&self) -> &[StructHandle] {
        &self.as_script().struct_handles
    }

    fn function_handles(&self) -> &[FunctionHandle] {
        &self.as_script().function_handles
    }

    fn function_instantiations(&self) -> &[FunctionInstantiation] {
        &self.as_script().function_instantiations
    }

    fn signatures(&self) -> &[Signature] {
        &self.as_script().signatures
    }

    fn constant_pool(&self) -> &[Constant] {
        &self.as_script().constant_pool
    }

    fn identifiers(&self) -> &[Identifier] {
        &self.as_script().identifiers
    }

    fn address_identifiers(&self) -> &[AccountAddress] {
        &self.as_script().address_identifiers
    }

    fn version(&self) -> u32 {
        self.as_script().version
    }

    fn code(&self) -> &CodeUnit {
        &self.as_script().code
    }

    fn immediate_dependencies(&self) -> Vec<ModuleId> {
        self.module_handles()
            .iter()
            .map(|handle| {
                ModuleId::new(
                    *self.address_identifier_at(handle.address),
                    self.identifier_at(handle.name).to_owned(),
                )
            })
            .collect()
    }
}

impl ModuleAccess for CompiledModule {
    fn as_module(&self) -> &CompiledModule {
        self
    }
}

impl ScriptAccess for CompiledScript {
    fn as_script(&self) -> &CompiledScript {
        self
    }
}
