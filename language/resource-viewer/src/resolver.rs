// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::module_cache::ModuleCache;
use anyhow::{anyhow, Result};
use libra_state_view::StateView;
use libra_types::{
    access_path::AccessPath,
    account_address::AccountAddress,
    language_storage::{ModuleId, StructTag, TypeTag},
};
use move_core_types::identifier::{IdentStr, Identifier};
use move_vm_types::loaded_data::types::{FatStructType, FatType};
use std::rc::Rc;
use stdlib::{stdlib_modules, StdLibOptions};
use vm::{
    access::ModuleAccess,
    file_format::{
        SignatureToken, StructDefinitionIndex, StructFieldInformation, StructHandleIndex,
    },
    CompiledModule,
};

pub(crate) struct Resolver<'a> {
    state: &'a dyn StateView,
    cache: ModuleCache,
}

impl<'a> Resolver<'a> {
    pub fn new(state: &'a dyn StateView, use_stdlib: bool) -> Self {
        let cache = ModuleCache::new();
        if use_stdlib {
            let modules = stdlib_modules(StdLibOptions::Staged);
            for module in modules {
                cache.insert(module.self_id(), module.clone().into_inner());
            }
        }
        Resolver { state, cache }
    }

    fn get_module(&self, address: &AccountAddress, name: &IdentStr) -> Result<Rc<CompiledModule>> {
        let module_id = ModuleId::new(*address, name.to_owned());
        if let Some(module) = self.cache.get(&module_id) {
            return Ok(module);
        }
        let blob = self
            .state
            .get(&AccessPath::code_access_path(&module_id))?
            .ok_or_else(|| anyhow!("Module {:?} can't be found", module_id))?;
        let compiled_module = CompiledModule::deserialize(&blob).map_err(|status| {
            anyhow!(
                "Module {:?} deserialize with error code {:?}",
                module_id,
                status
            )
        })?;
        Ok(self.cache.insert(module_id, compiled_module))
    }

    pub fn resolve_type(&self, type_tag: &TypeTag) -> Result<FatType> {
        Ok(match type_tag {
            TypeTag::Address => FatType::Address,
            TypeTag::Bool => FatType::Bool,
            TypeTag::Struct(st) => FatType::Struct(Box::new(self.resolve_struct(st)?)),
            TypeTag::U8 => FatType::U8,
            TypeTag::U64 => FatType::U64,
            TypeTag::U128 => FatType::U128,
            TypeTag::Vector(ty) => FatType::Vector(Box::new(self.resolve_type(ty)?)),
        })
    }

    pub fn resolve_struct(&self, struct_tag: &StructTag) -> Result<FatStructType> {
        let module = self.get_module(&struct_tag.address, &struct_tag.module)?;
        let struct_def = find_struct_def_in_module(module.clone(), struct_tag.name.as_ident_str())?;
        self.resolve_struct_definition(module, struct_def)
    }

    pub fn get_field_names(&self, ty: &FatStructType) -> Result<Vec<Identifier>> {
        let module = self.get_module(&ty.address, ty.module.as_ident_str())?;
        let struct_def_idx = find_struct_def_in_module(module.clone(), ty.name.as_ident_str())?;
        let struct_def = module.struct_def_at(struct_def_idx);

        match &struct_def.field_information {
            StructFieldInformation::Native => Err(anyhow!("Unexpected Native Struct")),
            StructFieldInformation::Declared(defs) => Ok(defs
                .iter()
                .map(|field_def| module.identifier_at(field_def.name).to_owned())
                .collect()),
        }
    }

    fn resolve_signature(
        &self,
        module: Rc<CompiledModule>,
        sig: &SignatureToken,
    ) -> Result<FatType> {
        Ok(match sig {
            SignatureToken::Bool => FatType::Bool,
            SignatureToken::U8 => FatType::U8,
            SignatureToken::U64 => FatType::U64,
            SignatureToken::U128 => FatType::U128,
            SignatureToken::Address => FatType::Address,
            SignatureToken::Vector(ty) => {
                FatType::Vector(Box::new(self.resolve_signature(module, ty)?))
            }
            SignatureToken::Struct(idx) => {
                FatType::Struct(Box::new(self.resolve_struct_handle(module, *idx)?))
            }
            SignatureToken::StructInstantiation(idx, toks) => {
                let struct_ty = self.resolve_struct_handle(module.clone(), *idx)?;
                let args = toks
                    .iter()
                    .map(|tok| self.resolve_signature(module.clone(), tok))
                    .collect::<Result<Vec<_>>>()?;
                FatType::Struct(Box::new(
                    struct_ty
                        .subst(&args)
                        .map_err(|status| anyhow!("Substitution failure: {:?}", status))?,
                ))
            }
            SignatureToken::TypeParameter(idx) => FatType::TyParam(*idx as usize),
            SignatureToken::MutableReference(_) | SignatureToken::Reference(_) => {
                return Err(anyhow!("Unexpected Reference"))
            }
        })
    }

    fn resolve_struct_handle(
        &self,
        module: Rc<CompiledModule>,
        idx: StructHandleIndex,
    ) -> Result<FatStructType> {
        let struct_handle = module.struct_handle_at(idx);
        let target_module = {
            let module_handle = module.module_handle_at(struct_handle.module);
            self.get_module(
                module.address_identifier_at(module_handle.address),
                module.identifier_at(module_handle.name),
            )?
        };
        let target_idx = find_struct_def_in_module(
            target_module.clone(),
            module.identifier_at(struct_handle.name),
        )?;
        self.resolve_struct_definition(target_module, target_idx)
    }

    fn resolve_struct_definition(
        &self,
        module: Rc<CompiledModule>,
        idx: StructDefinitionIndex,
    ) -> Result<FatStructType> {
        let struct_def = module.struct_def_at(idx);
        let struct_handle = module.struct_handle_at(struct_def.struct_handle);
        let address = *module.address();
        let module_name = module.name().to_owned();
        let name = module.identifier_at(struct_handle.name).to_owned();
        let is_resource = struct_handle.is_nominal_resource;
        let ty_args = (0..struct_handle.type_parameters.len())
            .map(FatType::TyParam)
            .collect();
        match &struct_def.field_information {
            StructFieldInformation::Native => Err(anyhow!("Unexpected Native Struct")),
            StructFieldInformation::Declared(defs) => Ok(FatStructType {
                address,
                module: module_name,
                name,
                is_resource,
                ty_args,
                layout: defs
                    .iter()
                    .map(|field_def| self.resolve_signature(module.clone(), &field_def.signature.0))
                    .collect::<Result<_>>()?,
            }),
        }
    }
}

fn find_struct_def_in_module(
    module: Rc<CompiledModule>,
    name: &IdentStr,
) -> Result<StructDefinitionIndex> {
    for (i, defs) in module.struct_defs().iter().enumerate() {
        let st_handle = module.struct_handle_at(defs.struct_handle);
        if module.identifier_at(st_handle.name) == name {
            return Ok(StructDefinitionIndex::new(i as u16));
        }
    }
    Err(anyhow!(
        "Struct {:?} not found in {:?}",
        name,
        module.self_id()
    ))
}
