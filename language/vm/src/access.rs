// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Defines accessors for compiled modules.

use crate::{
    file_format::{
        AddressPoolIndex, ByteArrayPoolIndex, CompiledModule, CompiledModuleMut, CompiledScript,
        FieldDefinition, FieldDefinitionIndex, FunctionDefinition, FunctionDefinitionIndex,
        FunctionHandle, FunctionHandleIndex, FunctionSignature, FunctionSignatureIndex,
        IdentifierIndex, LocalsSignature, LocalsSignatureIndex, MemberCount, ModuleHandle,
        ModuleHandleIndex, StructDefinition, StructDefinitionIndex, StructHandle,
        StructHandleIndex, TypeSignature, TypeSignatureIndex,
    },
    internals::ModuleIndex,
};
use libra_types::{
    account_address::AccountAddress,
    language_storage::ModuleId,
    vm_error::{StatusCode, VMStatus},
};
use move_core_types::identifier::{IdentStr, Identifier};

/// Represents accessors for a compiled module.
///
/// This is a trait to allow working across different wrappers for `CompiledModule`.
pub trait ModuleAccess: Sync {
    /// Returns the `CompiledModule` that will be used for accesses.
    fn as_module(&self) -> &CompiledModule;

    /// Returns the `ModuleHandle` for `self`.
    fn self_handle(&self) -> &ModuleHandle {
        self.module_handle_at(ModuleHandleIndex::new(
            CompiledModule::IMPLEMENTED_MODULE_INDEX,
        ))
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
        &self.as_module().as_inner().module_handles[idx.into_index()]
    }

    fn struct_handle_at(&self, idx: StructHandleIndex) -> &StructHandle {
        &self.as_module().as_inner().struct_handles[idx.into_index()]
    }

    fn function_handle_at(&self, idx: FunctionHandleIndex) -> &FunctionHandle {
        &self.as_module().as_inner().function_handles[idx.into_index()]
    }

    fn type_signature_at(&self, idx: TypeSignatureIndex) -> &TypeSignature {
        &self.as_module().as_inner().type_signatures[idx.into_index()]
    }

    fn function_signature_at(&self, idx: FunctionSignatureIndex) -> &FunctionSignature {
        &self.as_module().as_inner().function_signatures[idx.into_index()]
    }

    fn locals_signature_at(&self, idx: LocalsSignatureIndex) -> &LocalsSignature {
        &self.as_module().as_inner().locals_signatures[idx.into_index()]
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

    fn field_def_at(&self, idx: FieldDefinitionIndex) -> &FieldDefinition {
        &self.as_module().as_inner().field_defs[idx.into_index()]
    }

    fn function_def_at(&self, idx: FunctionDefinitionIndex) -> &FunctionDefinition {
        &self.as_module().as_inner().function_defs[idx.into_index()]
    }

    fn get_field_signature(&self, field_definition_index: FieldDefinitionIndex) -> &TypeSignature {
        let field_definition = self.field_def_at(field_definition_index);
        self.type_signature_at(field_definition.signature)
    }

    // XXX is a partial range required here?
    fn module_handles(&self) -> &[ModuleHandle] {
        &self.as_module().as_inner().module_handles
    }

    fn struct_handles(&self) -> &[StructHandle] {
        &self.as_module().as_inner().struct_handles
    }

    fn function_handles(&self) -> &[FunctionHandle] {
        &self.as_module().as_inner().function_handles
    }

    fn type_signatures(&self) -> &[TypeSignature] {
        &self.as_module().as_inner().type_signatures
    }

    fn function_signatures(&self) -> &[FunctionSignature] {
        &self.as_module().as_inner().function_signatures
    }

    fn locals_signatures(&self) -> &[LocalsSignature] {
        &self.as_module().as_inner().locals_signatures
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

    fn field_defs(&self) -> &[FieldDefinition] {
        &self.as_module().as_inner().field_defs
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

    fn field_def_range(
        &self,
        field_count: MemberCount,
        first_field: FieldDefinitionIndex,
    ) -> &[FieldDefinition] {
        let first_field = first_field.0 as usize;
        let field_count = field_count as usize;
        // Both `first_field` and `field_count` are `u16` before being converted to usize
        assume!(first_field <= usize::max_value() - field_count);
        let last_field = first_field + field_count;
        &self.as_module().as_inner().field_defs[first_field..last_field]
    }

    fn is_field_in_struct(
        &self,
        field_definition_index: FieldDefinitionIndex,
        struct_handle_index: StructHandleIndex,
    ) -> bool {
        let field_definition = self.field_def_at(field_definition_index);
        struct_handle_index == field_definition.struct_
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
        self.module_handle_at(ModuleHandleIndex::new(
            CompiledModule::IMPLEMENTED_MODULE_INDEX,
        ))
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

    fn type_signature_at(&self, idx: TypeSignatureIndex) -> &TypeSignature {
        &self.as_script().as_inner().type_signatures[idx.into_index()]
    }

    fn function_signature_at(&self, idx: FunctionSignatureIndex) -> &FunctionSignature {
        &self.as_script().as_inner().function_signatures[idx.into_index()]
    }

    fn locals_signature_at(&self, idx: LocalsSignatureIndex) -> &LocalsSignature {
        &self.as_script().as_inner().locals_signatures[idx.into_index()]
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

    fn module_handles(&self) -> &[ModuleHandle] {
        &self.as_script().as_inner().module_handles
    }

    fn struct_handles(&self) -> &[StructHandle] {
        &self.as_script().as_inner().struct_handles
    }

    fn function_handles(&self) -> &[FunctionHandle] {
        &self.as_script().as_inner().function_handles
    }

    fn type_signatures(&self) -> &[TypeSignature] {
        &self.as_script().as_inner().type_signatures
    }

    fn function_signatures(&self) -> &[FunctionSignature] {
        &self.as_script().as_inner().function_signatures
    }

    fn locals_signatures(&self) -> &[LocalsSignature] {
        &self.as_script().as_inner().locals_signatures
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

impl CompiledModuleMut {
    #[inline]
    pub(crate) fn check_field_range(
        &self,
        field_count: MemberCount,
        first_field: FieldDefinitionIndex,
    ) -> Option<VMStatus> {
        let first_field = first_field.into_index();
        let field_count = field_count as usize;
        // Both first_field and field_count are u16 so this is guaranteed to not overflow.
        // Note that last_field is exclusive, i.e. fields are in the range
        // [first_field, last_field).
        let last_field = first_field + field_count;
        if last_field > self.field_defs.len() {
            let msg = format!(
                "Field definition range [{},{}) out of range for {}",
                first_field,
                last_field,
                self.field_defs.len()
            );
            let status = VMStatus::new(StatusCode::RANGE_OUT_OF_BOUNDS).with_message(msg);
            Some(status)
        } else {
            None
        }
    }
}
