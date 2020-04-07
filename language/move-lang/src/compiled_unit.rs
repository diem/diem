// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    errors::*,
    expansion::ast::SpecId,
    hlir::ast as H,
    parser::ast::{FunctionName, ModuleIdent, Var},
    shared::unique_map::UniqueMap,
};
use bytecode_source_map::source_map::SourceMap;
use move_ir_types::location::*;
use move_vm::file_format as F;
use std::collections::BTreeMap;

//**************************************************************************************************
// Compiled Unit
//**************************************************************************************************

#[derive(Debug)]
pub struct SpecInfo {
    pub offset: F::CodeOffset,
    // Free locals that are used but not declared in the block
    pub used_locals: BTreeMap<Var, H::SingleType>,
}

#[derive(Debug)]
pub enum CompiledUnit {
    Module {
        ident: ModuleIdent,
        module: F::CompiledModule,
        source_map: SourceMap<Loc>,
        spec_info: UniqueMap<FunctionName, BTreeMap<SpecId, SpecInfo>>,
    },
    Script {
        loc: Loc,
        script: F::CompiledScript,
        source_map: SourceMap<Loc>,
        spec_info: BTreeMap<SpecId, SpecInfo>,
    },
}

impl CompiledUnit {
    pub fn name(&self) -> String {
        match self {
            CompiledUnit::Module { ident, .. } => format!("module_{}", &ident.0.value.name),
            CompiledUnit::Script { .. } => "script".into(),
        }
    }

    pub fn serialize(self) -> Vec<u8> {
        let mut serialized = Vec::<u8>::new();
        match self {
            CompiledUnit::Module { module, .. } => module.serialize(&mut serialized).unwrap(),
            CompiledUnit::Script { script, .. } => script.serialize(&mut serialized).unwrap(),
        };
        serialized
    }

    #[allow(dead_code)]
    pub fn serialize_debug(self) -> Vec<u8> {
        match self {
            CompiledUnit::Module { module, .. } => format!("{:?}", module),
            CompiledUnit::Script { script, .. } => format!("{:?}", script),
        }
        .into()
    }

    pub fn verify(self) -> (Self, Errors) {
        match self {
            CompiledUnit::Module {
                ident,
                module,
                source_map,
                spec_info,
            } => {
                let (module, errors) = verify_module(ident.loc(), module);
                let verified = CompiledUnit::Module {
                    ident,
                    module,
                    source_map,
                    spec_info,
                };
                (verified, errors)
            }
            CompiledUnit::Script {
                loc,
                script,
                source_map,
                spec_info,
            } => {
                let (script, errors) = verify_script(loc, script);
                let verified = CompiledUnit::Script {
                    loc,
                    script,
                    source_map,
                    spec_info,
                };
                (verified, errors)
            }
        }
    }
}

fn verify_module(loc: Loc, cm: F::CompiledModule) -> (F::CompiledModule, Errors) {
    match move_bytecode_verifier::verifier::VerifiedModule::new(cm) {
        Ok(v) => (v.into_inner(), vec![]),
        Err((cm, es)) => (
            cm,
            vec![vec![(
                loc,
                format!("ICE failed bytecode verifier: {:#?}", es),
            )]],
        ),
    }
}

fn verify_script(loc: Loc, cs: F::CompiledScript) -> (F::CompiledScript, Errors) {
    match move_bytecode_verifier::verifier::VerifiedScript::new(cs) {
        Ok(v) => (v.into_inner(), vec![]),
        Err((cs, es)) => (
            cs,
            vec![vec![(
                loc,
                format!("ICE failed bytecode verifier: {:#?}", es),
            )]],
        ),
    }
}

pub fn verify_units(units: Vec<CompiledUnit>) -> (Vec<CompiledUnit>, Errors) {
    let mut new_units = vec![];
    let mut errors = vec![];
    for unit in units {
        let (unit, mut es) = unit.verify();
        new_units.push(unit);
        errors.append(&mut es);
    }
    (new_units, errors)
}
