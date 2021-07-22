// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    diag,
    diagnostics::Diagnostics,
    expansion::ast::{ModuleIdent, ModuleIdent_, SpecId},
    hlir::ast as H,
    parser::ast::{FunctionName, ModuleName, Var},
    shared::{unique_map::UniqueMap, AddressBytes, Name},
};
use bytecode_source_map::source_map::SourceMap;
use move_binary_format::file_format as F;
use move_core_types::{
    account_address::AccountAddress, identifier::Identifier as MoveCoreIdentifier,
    language_storage::ModuleId,
};
use move_ir_types::location::*;
use move_symbol_pool::Symbol;
use std::collections::BTreeMap;

//**************************************************************************************************
// Compiled Unit
//**************************************************************************************************

#[derive(Debug, Clone)]
pub struct VarInfo {
    pub type_: H::SingleType,
    pub index: F::LocalIndex,
}

#[derive(Debug, Clone)]
pub struct SpecInfo {
    pub offset: F::CodeOffset,
    // Free locals that are used but not declared in the block
    pub used_locals: UniqueMap<Var, VarInfo>,
}

#[derive(Debug, Clone)]
pub struct FunctionInfo {
    pub spec_info: BTreeMap<SpecId, SpecInfo>,
    pub parameters: Vec<(Var, VarInfo)>,
}

#[derive(Debug, Clone)]
pub struct NamedCompiledModule {
    pub address_bytes: AddressBytes,
    pub name: Symbol,
    pub module: F::CompiledModule,
    pub source_map: SourceMap,
}

#[derive(Debug, Clone)]
pub struct NamedCompiledScript {
    pub name: Symbol,
    pub script: F::CompiledScript,
    pub source_map: SourceMap,
}

#[derive(Debug, Clone)]
pub struct AnnotatedCompiledModule {
    pub loc: Loc,
    pub module_name_loc: Loc,
    pub address_name: Option<Name>,
    pub named_module: NamedCompiledModule,
    pub function_infos: UniqueMap<FunctionName, FunctionInfo>,
}

#[derive(Debug, Clone)]
pub struct AnnotatedCompiledScript {
    pub loc: Loc,
    pub named_script: NamedCompiledScript,
    pub function_info: FunctionInfo,
}

pub trait TargetModule {}
pub trait TargetScript {}
impl TargetScript for AnnotatedCompiledScript {}
impl TargetScript for NamedCompiledScript {}
impl TargetModule for AnnotatedCompiledModule {}
impl TargetModule for NamedCompiledModule {}

#[derive(Debug, Clone)]
pub enum CompiledUnitEnum<TModule: TargetModule, TScript: TargetScript> {
    Module(TModule),
    Script(TScript),
}

pub type CompiledUnit = CompiledUnitEnum<NamedCompiledModule, NamedCompiledScript>;
pub type AnnotatedCompiledUnit = CompiledUnitEnum<AnnotatedCompiledModule, AnnotatedCompiledScript>;

impl AnnotatedCompiledModule {
    pub fn module_ident(&self) -> ModuleIdent {
        use crate::expansion::ast::Address;
        let address = match self.address_name {
            None => Address::Anonymous(sp(self.loc, self.named_module.address_bytes)),
            Some(n) => Address::Named(n),
        };
        sp(
            self.loc,
            ModuleIdent_::new(
                address,
                ModuleName(sp(self.module_name_loc, self.named_module.name)),
            ),
        )
    }

    pub fn module_id(&self) -> (Option<Name>, ModuleId) {
        let id = ModuleId::new(
            AccountAddress::new(self.named_module.address_bytes.into_bytes()),
            MoveCoreIdentifier::new(self.named_module.name.to_string()).unwrap(),
        );
        (self.address_name, id)
    }
}

impl AnnotatedCompiledUnit {
    pub fn verify(self) -> (Self, Diagnostics) {
        match self {
            Self::Module(AnnotatedCompiledModule {
                loc,
                module_name_loc,
                address_name,
                named_module: module,
                function_infos,
            }) => {
                let NamedCompiledModule {
                    address_bytes,
                    name,
                    module,
                    source_map,
                } = module;
                let (module, errors) = verify_module(loc, module);
                let verified = AnnotatedCompiledModule {
                    loc,
                    module_name_loc,
                    address_name,
                    named_module: NamedCompiledModule {
                        address_bytes,
                        name,
                        module,
                        source_map,
                    },
                    function_infos,
                };
                (Self::Module(verified), errors)
            }
            Self::Script(AnnotatedCompiledScript {
                loc,
                named_script: script,
                function_info,
            }) => {
                let NamedCompiledScript {
                    name,
                    script,
                    source_map,
                } = script;
                let (script, errors) = verify_script(loc, script);
                let verified = AnnotatedCompiledScript {
                    named_script: NamedCompiledScript {
                        name,
                        script,
                        source_map,
                    },
                    loc,
                    function_info,
                };
                (Self::Script(verified), errors)
            }
        }
    }

    pub fn into_compiled_unit(self) -> CompiledUnit {
        match self {
            Self::Module(AnnotatedCompiledModule {
                named_module: module,
                ..
            }) => CompiledUnitEnum::Module(module),
            Self::Script(AnnotatedCompiledScript {
                named_script: script,
                ..
            }) => CompiledUnitEnum::Script(script),
        }
    }

    pub fn loc(&self) -> &Loc {
        match self {
            Self::Module(AnnotatedCompiledModule { loc, .. })
            | Self::Script(AnnotatedCompiledScript { loc, .. }) => loc,
        }
    }
}

impl CompiledUnit {
    pub fn name(&self) -> Symbol {
        match self {
            Self::Module(NamedCompiledModule { name, .. })
            | Self::Script(NamedCompiledScript { name, .. }) => *name,
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut serialized = Vec::<u8>::new();
        match self {
            Self::Module(NamedCompiledModule { module, .. }) => {
                module.serialize(&mut serialized).unwrap()
            }
            Self::Script(NamedCompiledScript { script, .. }) => {
                script.serialize(&mut serialized).unwrap()
            }
        };
        serialized
    }

    #[allow(dead_code)]
    pub fn serialize_debug(self) -> Vec<u8> {
        match self {
            Self::Module(NamedCompiledModule { module, .. }) => format!("{:?}", module),
            Self::Script(NamedCompiledScript { script, .. }) => format!("{:?}", script),
        }
        .into()
    }

    pub fn serialize_source_map(&self) -> Vec<u8> {
        match self {
            Self::Module(NamedCompiledModule { source_map, .. }) => {
                bcs::to_bytes(source_map).unwrap()
            }
            Self::Script(NamedCompiledScript { source_map, .. }) => {
                bcs::to_bytes(source_map).unwrap()
            }
        }
    }
}

fn verify_module(loc: Loc, cm: F::CompiledModule) -> (F::CompiledModule, Diagnostics) {
    match move_bytecode_verifier::verifier::verify_module(&cm) {
        Ok(_) => (cm, Diagnostics::new()),
        Err(e) => (
            cm,
            Diagnostics::from(vec![diag!(
                Bug::BytecodeVerification,
                (loc, format!("ICE failed bytecode verifier: {:#?}", e)),
            )]),
        ),
    }
}

fn verify_script(loc: Loc, cs: F::CompiledScript) -> (F::CompiledScript, Diagnostics) {
    match move_bytecode_verifier::verifier::verify_script(&cs) {
        Ok(_) => (cs, Diagnostics::new()),
        Err(e) => (
            cs,
            Diagnostics::from(vec![diag!(
                Bug::BytecodeVerification,
                (loc, format!("ICE failed bytecode verifier: {:#?}", e)),
            )]),
        ),
    }
}

pub fn verify_units(
    units: Vec<AnnotatedCompiledUnit>,
) -> (Vec<AnnotatedCompiledUnit>, Diagnostics) {
    let mut new_units = vec![];
    let mut diags = Diagnostics::new();
    for unit in units {
        let (unit, ds) = unit.verify();
        new_units.push(unit);
        diags.extend(ds);
    }
    (new_units, diags)
}
