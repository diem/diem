// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module implements a resolver for importing a SignatureToken defined in one module into
//! another. This functionaliy is used in verify_module_dependencies and verify_script_dependencies.
use libra_types::{
    account_address::AccountAddress,
    vm_error::{StatusCode, VMStatus},
};
use move_core_types::identifier::Identifier;
use std::collections::BTreeMap;
use vm::{
    access::ModuleAccess,
    file_format::{
        AddressPoolIndex, IdentifierIndex, ModuleHandle, ModuleHandleIndex, Signature,
        SignatureToken, StructHandle, StructHandleIndex,
    },
};

/// Resolution context for importing types
pub struct Resolver {
    address_map: BTreeMap<AccountAddress, AddressPoolIndex>,
    identifier_map: BTreeMap<Identifier, IdentifierIndex>,
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
        let mut identifier_map = BTreeMap::new();
        for (idx, name) in module.identifiers().iter().enumerate() {
            identifier_map.insert(name.clone(), IdentifierIndex(idx as u16));
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
            identifier_map,
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
    ) -> Result<SignatureToken, VMStatus> {
        match sig_token {
            SignatureToken::Bool
            | SignatureToken::U8
            | SignatureToken::U64
            | SignatureToken::U128
            | SignatureToken::Address
            | SignatureToken::TypeParameter(_) => Ok(sig_token.clone()),
            SignatureToken::Vector(ty) => Ok(SignatureToken::Vector(Box::new(
                self.import_signature_token(dependency, ty)?,
            ))),
            SignatureToken::Struct(sh_idx) => {
                let struct_handle = dependency.struct_handle_at(*sh_idx);
                let defining_module_handle = dependency.module_handle_at(struct_handle.module);
                let defining_module_address = dependency.address_at(defining_module_handle.address);
                let defining_module_name = dependency.identifier_at(defining_module_handle.name);
                let local_module_handle = ModuleHandle {
                    address: *self
                        .address_map
                        .get(defining_module_address)
                        .ok_or_else(|| VMStatus::new(StatusCode::TYPE_RESOLUTION_FAILURE))?,
                    name: *self
                        .identifier_map
                        .get(defining_module_name)
                        .ok_or_else(|| VMStatus::new(StatusCode::TYPE_RESOLUTION_FAILURE))?,
                };
                let struct_name = dependency.identifier_at(struct_handle.name);
                let local_struct_handle = StructHandle {
                    module: *self
                        .module_handle_map
                        .get(&local_module_handle)
                        .ok_or_else(|| VMStatus::new(StatusCode::TYPE_RESOLUTION_FAILURE))?,
                    name: *self
                        .identifier_map
                        .get(struct_name)
                        .ok_or_else(|| VMStatus::new(StatusCode::TYPE_RESOLUTION_FAILURE))?,
                    is_nominal_resource: struct_handle.is_nominal_resource,
                    type_parameters: struct_handle.type_parameters.clone(),
                };
                Ok(SignatureToken::Struct(
                    *self
                        .struct_handle_map
                        .get(&local_struct_handle)
                        .ok_or_else(|| VMStatus::new(StatusCode::TYPE_RESOLUTION_FAILURE))?,
                ))
            }
            SignatureToken::StructInstantiation(sh_idx, type_args) => {
                let struct_handle = dependency.struct_handle_at(*sh_idx);
                let defining_module_handle = dependency.module_handle_at(struct_handle.module);
                let defining_module_address = dependency.address_at(defining_module_handle.address);
                let defining_module_name = dependency.identifier_at(defining_module_handle.name);
                let local_module_handle = ModuleHandle {
                    address: *self
                        .address_map
                        .get(defining_module_address)
                        .ok_or_else(|| VMStatus::new(StatusCode::TYPE_RESOLUTION_FAILURE))?,
                    name: *self
                        .identifier_map
                        .get(defining_module_name)
                        .ok_or_else(|| VMStatus::new(StatusCode::TYPE_RESOLUTION_FAILURE))?,
                };
                let struct_name = dependency.identifier_at(struct_handle.name);
                let local_struct_handle = StructHandle {
                    module: *self
                        .module_handle_map
                        .get(&local_module_handle)
                        .ok_or_else(|| VMStatus::new(StatusCode::TYPE_RESOLUTION_FAILURE))?,
                    name: *self
                        .identifier_map
                        .get(struct_name)
                        .ok_or_else(|| VMStatus::new(StatusCode::TYPE_RESOLUTION_FAILURE))?,
                    is_nominal_resource: struct_handle.is_nominal_resource,
                    type_parameters: struct_handle.type_parameters.clone(),
                };
                Ok(SignatureToken::StructInstantiation(
                    *self
                        .struct_handle_map
                        .get(&local_struct_handle)
                        .ok_or_else(|| VMStatus::new(StatusCode::TYPE_RESOLUTION_FAILURE))?,
                    type_args
                        .iter()
                        .map(|t| self.import_signature_token(dependency, &t))
                        .collect::<Result<Vec<_>, VMStatus>>()?,
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

    pub fn import_signature(
        &self,
        signature: &Signature,
        dependency: &impl ModuleAccess,
    ) -> Result<Signature, VMStatus> {
        let mut imported_signature = Vec::<SignatureToken>::new();
        for e in &signature.0 {
            imported_signature.push(self.import_signature_token(dependency, e)?);
        }
        Ok(Signature(imported_signature))
    }

    pub fn compare_cross_module_signatures(
        &self,
        handle_sig: &Signature,
        def_sig: &Signature,
        dependency: &impl ModuleAccess,
    ) -> Result<(), VMStatus> {
        let imported_signature = self.import_signature(def_sig, dependency)?;
        if &imported_signature == handle_sig {
            Ok(())
        } else {
            Err(VMStatus::new(StatusCode::TYPE_MISMATCH))
        }
    }
}
