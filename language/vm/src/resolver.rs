// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module implements a resolver for importing a SignatureToken defined in one module into
//! another. This functionaliy is used in verify_module_dependencies and verify_script_dependencies.
use crate::{
    access::ModuleAccess,
    errors::VMStaticViolation,
    file_format::{
        AddressPoolIndex, FunctionSignature, ModuleHandle, ModuleHandleIndex, SignatureToken,
        StringPoolIndex, StructHandle, StructHandleIndex,
    },
};
use std::collections::BTreeMap;
use types::account_address::AccountAddress;

/// Resolution context for importing types
pub struct Resolver {
    address_map: BTreeMap<AccountAddress, AddressPoolIndex>,
    string_map: BTreeMap<String, StringPoolIndex>,
    module_handle_map: BTreeMap<ModuleHandle, ModuleHandleIndex>,
    struct_handle_map: BTreeMap<StructHandle, StructHandleIndex>,
}

impl Resolver {
    /// create a new instance of Resolver for module
    pub fn new(module: &impl ModuleAccess) -> Self {
        let mut address_map = BTreeMap::new();
        for (idx, address) in module.address_pool().iter().enumerate() {
            address_map.insert(address.clone(), AddressPoolIndex(idx as u16));
        }
        let mut string_map = BTreeMap::new();
        for (idx, name) in module.string_pool().iter().enumerate() {
            string_map.insert(name.clone(), StringPoolIndex(idx as u16));
        }
        let mut module_handle_map = BTreeMap::new();
        for (idx, module_hadndle) in module.module_handles().iter().enumerate() {
            module_handle_map.insert(module_hadndle.clone(), ModuleHandleIndex(idx as u16));
        }
        let mut struct_handle_map = BTreeMap::new();
        for (idx, struct_handle) in module.struct_handles().iter().enumerate() {
            struct_handle_map.insert(struct_handle.clone(), StructHandleIndex(idx as u16));
        }
        Self {
            address_map,
            string_map,
            module_handle_map,
            struct_handle_map,
        }
    }

    /// given a signature token in dependency, construct an equivalent signature token in the
    /// context of this resolver and return it; return an error if resolution fails
    pub fn import_signature_token(
        &self,
        dependency: &impl ModuleAccess,
        sig_token: &SignatureToken,
    ) -> Result<SignatureToken, VMStaticViolation> {
        match sig_token {
            SignatureToken::Bool
            | SignatureToken::U64
            | SignatureToken::String
            | SignatureToken::ByteArray
            | SignatureToken::Address
            | SignatureToken::TypeParameter(_) => Ok(sig_token.clone()),
            SignatureToken::Struct(sh_idx, types) => {
                let struct_handle = dependency.struct_handle_at(*sh_idx);
                let defining_module_handle = dependency.module_handle_at(struct_handle.module);
                let defining_module_address = dependency.address_at(defining_module_handle.address);
                let defining_module_name = dependency.string_at(defining_module_handle.name);
                let local_module_handle = ModuleHandle {
                    address: *self
                        .address_map
                        .get(defining_module_address)
                        .ok_or(VMStaticViolation::TypeResolutionFailure)?,
                    name: *self
                        .string_map
                        .get(defining_module_name)
                        .ok_or(VMStaticViolation::TypeResolutionFailure)?,
                };
                let struct_name = dependency.string_at(struct_handle.name);
                let local_struct_handle = StructHandle {
                    module: *self
                        .module_handle_map
                        .get(&local_module_handle)
                        .ok_or(VMStaticViolation::TypeResolutionFailure)?,
                    name: *self
                        .string_map
                        .get(struct_name)
                        .ok_or(VMStaticViolation::TypeResolutionFailure)?,
                    is_nominal_resource: struct_handle.is_nominal_resource,
                    type_formals: struct_handle.type_formals.clone(),
                };
                Ok(SignatureToken::Struct(
                    *self
                        .struct_handle_map
                        .get(&local_struct_handle)
                        .ok_or(VMStaticViolation::TypeResolutionFailure)?,
                    types
                        .iter()
                        .map(|t| self.import_signature_token(dependency, &t))
                        .collect::<Result<Vec<_>, VMStaticViolation>>()?,
                ))
            }
            SignatureToken::Reference(sub_sig_token) => Ok(SignatureToken::Reference(Box::new(
                self.import_signature_token(dependency, sub_sig_token)?,
            ))),
            SignatureToken::MutableReference(sub_sig_token) => {
                Ok(SignatureToken::MutableReference(Box::new(
                    self.import_signature_token(dependency, sub_sig_token)?,
                )))
            }
        }
    }

    /// given a function signature in dependency, construct an equivalent function signature in the
    /// context of this resolver and return it; return an error if resolution fails
    pub fn import_function_signature(
        &self,
        dependency: &impl ModuleAccess,
        func_sig: &FunctionSignature,
    ) -> Result<FunctionSignature, VMStaticViolation> {
        let mut return_types = Vec::<SignatureToken>::new();
        let mut arg_types = Vec::<SignatureToken>::new();
        for e in &func_sig.return_types {
            return_types.push(self.import_signature_token(dependency, e)?);
        }
        for e in &func_sig.arg_types {
            arg_types.push(self.import_signature_token(dependency, e)?);
        }
        Ok(FunctionSignature {
            return_types,
            arg_types,
            type_formals: func_sig.type_formals.clone(),
        })
    }
}
