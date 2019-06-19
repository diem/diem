// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Defines accessors for compiled modules.

use std::slice;

use types::{account_address::AccountAddress, byte_array::ByteArray, language_storage::CodeKey};

use crate::{
    errors::VMStaticViolation,
    file_format::{
        AddressPoolIndex, ByteArrayPoolIndex, CompiledModule, CompiledModuleMut, CompiledScript,
        FieldDefinition, FieldDefinitionIndex, FunctionDefinition, FunctionDefinitionIndex,
        FunctionHandle, FunctionHandleIndex, FunctionSignature, FunctionSignatureIndex,
        LocalsSignature, LocalsSignatureIndex, MemberCount, ModuleHandle, ModuleHandleIndex,
        StringPoolIndex, StructDefinition, StructDefinitionIndex, StructHandle, StructHandleIndex,
        TypeSignature, TypeSignatureIndex,
    },
    internals::ModuleIndex,
    IndexKind,
};

/// Represents accessors common to modules and scripts.
///
/// This is done as a trait because in the future, we may be able to write an alternative impl for
/// bytecode that's already been checked for internal consistency.
pub trait BaseAccess: Sync {
    fn module_handle_at(&self, idx: ModuleHandleIndex) -> &ModuleHandle;
    fn struct_handle_at(&self, idx: StructHandleIndex) -> &StructHandle;
    fn function_handle_at(&self, idx: FunctionHandleIndex) -> &FunctionHandle;

    fn type_signature_at(&self, idx: TypeSignatureIndex) -> &TypeSignature;
    fn function_signature_at(&self, idx: FunctionSignatureIndex) -> &FunctionSignature;
    fn locals_signature_at(&self, idx: LocalsSignatureIndex) -> &LocalsSignature;

    fn string_at(&self, idx: StringPoolIndex) -> &str;
    fn byte_array_at(&self, idx: ByteArrayPoolIndex) -> &ByteArray;
    fn address_at(&self, idx: AddressPoolIndex) -> &AccountAddress;

    // XXX is a partial range required here?
    fn module_handles(&self) -> slice::Iter<ModuleHandle>;
    fn struct_handles(&self) -> slice::Iter<StructHandle>;
    fn function_handles(&self) -> slice::Iter<FunctionHandle>;

    fn type_signatures(&self) -> slice::Iter<TypeSignature>;
    fn function_signatures(&self) -> slice::Iter<FunctionSignature>;
    fn locals_signatures(&self) -> slice::Iter<LocalsSignature>;

    fn byte_array_pool(&self) -> slice::Iter<ByteArray>;
    fn address_pool(&self) -> slice::Iter<AccountAddress>;
    fn string_pool(&self) -> slice::Iter<String>;
}

/// Represents accessors for a compiled script.
///
/// This is done as a trait because in the future, we may be able to write an alternative impl for a
/// script that's already been checked for internal consistency.
pub trait ScriptAccess: BaseAccess {
    fn main(&self) -> &FunctionDefinition;
}

/// Represents accessors for a compiled module.
///
/// This is done as a trait because in the future, we may be able to write an alternative impl for a
/// module that's already been checked for internal consistency.
pub trait ModuleAccess: BaseAccess {
    fn struct_def_at(&self, idx: StructDefinitionIndex) -> &StructDefinition;
    fn field_def_at(&self, idx: FieldDefinitionIndex) -> &FieldDefinition;
    fn function_def_at(&self, idx: FunctionDefinitionIndex) -> &FunctionDefinition;

    fn struct_defs(&self) -> slice::Iter<StructDefinition>;
    fn field_defs(&self) -> slice::Iter<FieldDefinition>;
    fn function_defs(&self) -> slice::Iter<FunctionDefinition>;

    fn code_key_for_handle(&self, module_handle_idx: &ModuleHandle) -> CodeKey;
    fn self_code_key(&self) -> CodeKey;

    fn field_def_range(
        &self,
        field_count: MemberCount,
        first_field: FieldDefinitionIndex,
    ) -> slice::Iter<FieldDefinition>;
}

macro_rules! impl_base_access {
    ($ty:ty) => {
        impl BaseAccess for $ty {
            fn module_handle_at(&self, idx: ModuleHandleIndex) -> &ModuleHandle {
                &self.as_inner().module_handles[idx.into_index()]
            }

            fn struct_handle_at(&self, idx: StructHandleIndex) -> &StructHandle {
                &self.as_inner().struct_handles[idx.into_index()]
            }

            fn function_handle_at(&self, idx: FunctionHandleIndex) -> &FunctionHandle {
                &self.as_inner().function_handles[idx.into_index()]
            }

            fn type_signature_at(&self, idx: TypeSignatureIndex) -> &TypeSignature {
                &self.as_inner().type_signatures[idx.into_index()]
            }

            fn function_signature_at(&self, idx: FunctionSignatureIndex) -> &FunctionSignature {
                &self.as_inner().function_signatures[idx.into_index()]
            }

            fn locals_signature_at(&self, idx: LocalsSignatureIndex) -> &LocalsSignature {
                &self.as_inner().locals_signatures[idx.into_index()]
            }

            fn string_at(&self, idx: StringPoolIndex) -> &str {
                self.as_inner().string_pool[idx.into_index()].as_str()
            }

            fn byte_array_at(&self, idx: ByteArrayPoolIndex) -> &ByteArray {
                &self.as_inner().byte_array_pool[idx.into_index()]
            }

            fn address_at(&self, idx: AddressPoolIndex) -> &AccountAddress {
                &self.as_inner().address_pool[idx.into_index()]
            }

            fn module_handles(&self) -> slice::Iter<ModuleHandle> {
                self.as_inner().module_handles[..].iter()
            }
            fn struct_handles(&self) -> slice::Iter<StructHandle> {
                self.as_inner().struct_handles[..].iter()
            }
            fn function_handles(&self) -> slice::Iter<FunctionHandle> {
                self.as_inner().function_handles[..].iter()
            }

            fn type_signatures(&self) -> slice::Iter<TypeSignature> {
                self.as_inner().type_signatures[..].iter()
            }
            fn function_signatures(&self) -> slice::Iter<FunctionSignature> {
                self.as_inner().function_signatures[..].iter()
            }
            fn locals_signatures(&self) -> slice::Iter<LocalsSignature> {
                self.as_inner().locals_signatures[..].iter()
            }

            fn byte_array_pool(&self) -> slice::Iter<ByteArray> {
                self.as_inner().byte_array_pool[..].iter()
            }
            fn address_pool(&self) -> slice::Iter<AccountAddress> {
                self.as_inner().address_pool[..].iter()
            }
            fn string_pool(&self) -> slice::Iter<String> {
                self.as_inner().string_pool[..].iter()
            }
        }
    };
}

impl_base_access!(CompiledModule);
impl_base_access!(CompiledScript);

impl ModuleAccess for CompiledModule {
    fn self_code_key(&self) -> CodeKey {
        self.self_code_key()
    }

    fn code_key_for_handle(&self, module_handle: &ModuleHandle) -> CodeKey {
        self.code_key_for_handle(module_handle)
    }

    fn struct_def_at(&self, idx: StructDefinitionIndex) -> &StructDefinition {
        &self.as_inner().struct_defs[idx.into_index()]
    }

    fn field_def_at(&self, idx: FieldDefinitionIndex) -> &FieldDefinition {
        &self.as_inner().field_defs[idx.into_index()]
    }

    fn function_def_at(&self, idx: FunctionDefinitionIndex) -> &FunctionDefinition {
        &self.as_inner().function_defs[idx.into_index()]
    }

    fn struct_defs(&self) -> slice::Iter<StructDefinition> {
        self.as_inner().struct_defs[..].iter()
    }

    fn field_defs(&self) -> slice::Iter<FieldDefinition> {
        self.as_inner().field_defs[..].iter()
    }

    fn function_defs(&self) -> slice::Iter<FunctionDefinition> {
        self.as_inner().function_defs[..].iter()
    }

    fn field_def_range(
        &self,
        field_count: MemberCount,
        first_field: FieldDefinitionIndex,
    ) -> slice::Iter<FieldDefinition> {
        let first_field = first_field.0 as usize;
        let field_count = field_count as usize;
        let last_field = first_field + field_count;
        self.as_inner().field_defs[first_field..last_field].iter()
    }
}

impl ScriptAccess for CompiledScript {
    fn main(&self) -> &FunctionDefinition {
        &self.as_inner().main
    }
}

impl CompiledModuleMut {
    #[inline]
    pub(crate) fn check_field_range(
        &self,
        field_count: MemberCount,
        first_field: FieldDefinitionIndex,
    ) -> Option<VMStaticViolation> {
        let first_field = first_field.into_index();
        let field_count = field_count as usize;
        // Both first_field and field_count are u16 so this is guaranteed to not overflow.
        // Note that last_field is exclusive, i.e. fields are in the range
        // [first_field, last_field).
        let last_field = first_field + field_count;
        if last_field > self.field_defs.len() {
            Some(VMStaticViolation::RangeOutOfBounds(
                IndexKind::FieldDefinition,
                self.field_defs.len(),
                first_field,
                last_field,
            ))
        } else {
            None
        }
    }
}
