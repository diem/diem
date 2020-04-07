// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Defines accessors for compiled modules.

use crate::{file_format::*, internals::ModuleIndex};
use libra_types::{account_address::AccountAddress, language_storage::ModuleId};
use move_core_types::identifier::{IdentStr, Identifier};

/// Represents accessors for a compiled module.
///
/// This is a trait to allow working across different wrappers for `CompiledModule`.
pub trait ModuleAccess: Sync {
    /// Returns the `CompiledModule` that will be used for accesses.
    fn as_module(&self) -> &CompiledModule;

    /// Returns the `ModuleHandle` for `self`.
    fn self_handle(&self) -> &ModuleHandle {
        assume_preconditions!(); // invariant
        let handle =
            self.module_handle_at(ModuleHandleIndex(CompiledModule::IMPLEMENTED_MODULE_INDEX));
        assumed_postcondition!(
            handle.address.into_index() < self.as_module().as_inner().address_pool.len()
        ); // invariant
        assumed_postcondition!(
            handle.name.into_index() < self.as_module().as_inner().identifiers.len()
        ); // invariant
        handle
    }

    /// Returns the name of the module.
    fn name(&self) -> &IdentStr {
        self.identifier_at(self.self_handle().name)
    }

    /// Returns the address of the module.
    fn address(&self) -> &AccountAddress {
        self.address_at(self.self_handle().address)
    }

    fn module_handle_at(&self, idx: ModuleHandleIndex) -> &ModuleHandle {
        let handle = &self.as_module().as_inner().module_handles[idx.into_index()];
        assumed_postcondition!(
            handle.address.into_index() < self.as_module().as_inner().address_pool.len()
        ); // invariant
        assumed_postcondition!(
            handle.name.into_index() < self.as_module().as_inner().identifiers.len()
        ); // invariant
        handle
    }

    fn struct_handle_at(&self, idx: StructHandleIndex) -> &StructHandle {
        let handle = &self.as_module().as_inner().struct_handles[idx.into_index()];
        assumed_postcondition!(
            handle.module.into_index() < self.as_module().as_inner().module_handles.len()
        ); // invariant
        handle
    }

    fn function_handle_at(&self, idx: FunctionHandleIndex) -> &FunctionHandle {
        let handle = &self.as_module().as_inner().function_handles[idx.into_index()];
        assumed_postcondition!(
            handle.parameters.into_index() < self.as_module().as_inner().signatures.len()
        ); // invariant
        assumed_postcondition!(
            handle.return_.into_index() < self.as_module().as_inner().signatures.len()
        ); // invariant
        handle
    }

    fn field_handle_at(&self, idx: FieldHandleIndex) -> &FieldHandle {
        let handle = &self.as_module().as_inner().field_handles[idx.into_index()];
        assumed_postcondition!(
            handle.owner.into_index() < self.as_module().as_inner().struct_defs.len()
        ); // invariant
        handle
    }

    fn struct_instantiation_at(&self, idx: StructDefInstantiationIndex) -> &StructDefInstantiation {
        &self.as_module().as_inner().struct_def_instantiations[idx.into_index()]
    }

    fn function_instantiation_at(&self, idx: FunctionInstantiationIndex) -> &FunctionInstantiation {
        &self.as_module().as_inner().function_instantiations[idx.into_index()]
    }

    fn field_instantiation_at(&self, idx: FieldInstantiationIndex) -> &FieldInstantiation {
        &self.as_module().as_inner().field_instantiations[idx.into_index()]
    }

    fn signature_at(&self, idx: SignatureIndex) -> &Signature {
        &self.as_module().as_inner().signatures[idx.into_index()]
    }

    fn identifier_at(&self, idx: IdentifierIndex) -> &IdentStr {
        &self.as_module().as_inner().identifiers[idx.into_index()]
    }

    fn byte_array_at(&self, idx: ByteArrayPoolIndex) -> &[u8] {
        self.as_module().as_inner().byte_array_pool[idx.into_index()].as_slice()
    }

    fn address_at(&self, idx: AddressPoolIndex) -> &AccountAddress {
        &self.as_module().as_inner().address_pool[idx.into_index()]
    }

    fn struct_def_at(&self, idx: StructDefinitionIndex) -> &StructDefinition {
        &self.as_module().as_inner().struct_defs[idx.into_index()]
    }

    fn function_def_at(&self, idx: FunctionDefinitionIndex) -> &FunctionDefinition {
        let result = &self.as_module().as_inner().function_defs[idx.into_index()];
        assumed_postcondition!(result.function.into_index() < self.function_handles().len()); // invariant
        assumed_postcondition!(result.code.locals.into_index() < self.signatures().len()); // invariant
        result
    }

    fn module_handles(&self) -> &[ModuleHandle] {
        &self.as_module().as_inner().module_handles
    }

    fn struct_handles(&self) -> &[StructHandle] {
        &self.as_module().as_inner().struct_handles
    }

    fn function_handles(&self) -> &[FunctionHandle] {
        &self.as_module().as_inner().function_handles
    }

    fn field_handles(&self) -> &[FieldHandle] {
        &self.as_module().as_inner().field_handles
    }

    fn struct_instantiations(&self) -> &[StructDefInstantiation] {
        &self.as_module().as_inner().struct_def_instantiations
    }

    fn function_instantiations(&self) -> &[FunctionInstantiation] {
        &self.as_module().as_inner().function_instantiations
    }

    fn field_instantiations(&self) -> &[FieldInstantiation] {
        &self.as_module().as_inner().field_instantiations
    }

    fn signatures(&self) -> &[Signature] {
        &self.as_module().as_inner().signatures
    }

    fn byte_array_pool(&self) -> &[Vec<u8>] {
        &self.as_module().as_inner().byte_array_pool
    }

    fn address_pool(&self) -> &[AccountAddress] {
        &self.as_module().as_inner().address_pool
    }

    fn identifiers(&self) -> &[Identifier] {
        &self.as_module().as_inner().identifiers
    }

    fn struct_defs(&self) -> &[StructDefinition] {
        &self.as_module().as_inner().struct_defs
    }

    fn function_defs(&self) -> &[FunctionDefinition] {
        &self.as_module().as_inner().function_defs
    }

    fn module_id_for_handle(&self, module_handle_idx: &ModuleHandle) -> ModuleId {
        self.as_module().module_id_for_handle(module_handle_idx)
    }

    fn self_id(&self) -> ModuleId {
        self.as_module().self_id()
    }
}

/// Represents accessors for a compiled script.
///
/// This is a trait to allow working across different wrappers for `CompiledScript`.
pub trait ScriptAccess: Sync {
    /// Returns the `CompiledScript` that will be used for accesses.
    fn as_script(&self) -> &CompiledScript;

    /// Returns the `ModuleHandle` for `self`.
    fn self_handle(&self) -> &ModuleHandle {
        self.module_handle_at(ModuleHandleIndex(CompiledModule::IMPLEMENTED_MODULE_INDEX))
    }

    fn module_handle_at(&self, idx: ModuleHandleIndex) -> &ModuleHandle {
        &self.as_script().as_inner().module_handles[idx.into_index()]
    }

    fn struct_handle_at(&self, idx: StructHandleIndex) -> &StructHandle {
        &self.as_script().as_inner().struct_handles[idx.into_index()]
    }

    fn function_handle_at(&self, idx: FunctionHandleIndex) -> &FunctionHandle {
        &self.as_script().as_inner().function_handles[idx.into_index()]
    }

    fn signature_at(&self, idx: SignatureIndex) -> &Signature {
        &self.as_script().as_inner().signatures[idx.into_index()]
    }

    fn identifier_at(&self, idx: IdentifierIndex) -> &IdentStr {
        &self.as_script().as_inner().identifiers[idx.into_index()]
    }

    fn byte_array_at(&self, idx: ByteArrayPoolIndex) -> &[u8] {
        self.as_script().as_inner().byte_array_pool[idx.into_index()].as_slice()
    }

    fn address_at(&self, idx: AddressPoolIndex) -> &AccountAddress {
        &self.as_script().as_inner().address_pool[idx.into_index()]
    }

    fn function_instantiation_at(&self, idx: FunctionInstantiationIndex) -> &FunctionInstantiation {
        &self.as_script().as_inner().function_instantiations[idx.into_index()]
    }

    fn module_handles(&self) -> &[ModuleHandle] {
        &self.as_script().as_inner().module_handles
    }

    fn struct_handles(&self) -> &[StructHandle] {
        &self.as_script().as_inner().struct_handles
    }

    fn function_handles(&self) -> &[FunctionHandle] {
        &self.as_script().as_inner().function_handles
    }

    fn function_instantiations(&self) -> &[FunctionInstantiation] {
        &self.as_script().as_inner().function_instantiations
    }

    fn signatures(&self) -> &[Signature] {
        &self.as_script().as_inner().signatures
    }

    fn byte_array_pool(&self) -> &[Vec<u8>] {
        &self.as_script().as_inner().byte_array_pool
    }

    fn address_pool(&self) -> &[AccountAddress] {
        &self.as_script().as_inner().address_pool
    }

    fn identifiers(&self) -> &[Identifier] {
        &self.as_script().as_inner().identifiers
    }

    fn main(&self) -> &FunctionDefinition {
        &self.as_script().as_inner().main
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
