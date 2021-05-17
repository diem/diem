// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Merges specification modules into their target modules.
//!
//! There are some issues with this approach which we may want to fix down the road:
//! - If a spec module contains a `use`, we don't want the target module be able to use it.
//! - Similarly, we also *may* want the spec module not be able to see target module `use`
//!   declarations, and require it to repeat them.
//! A solution to both problems can be to mark names introduced by `use` to whether they
//! are for specs or not, and allow the older to resolve only in spec contexts.

use crate::{
    parser::ast::{Definition, ModuleDefinition, ModuleMember, Program},
    shared::*,
};
use std::collections::BTreeMap;

/// Given a parsed program, merge all specification modules into their target modules.
pub fn program(compilation_env: &mut CompilationEnv, prog: Program) -> Program {
    let Program {
        source_definitions,
        lib_definitions,
    } = prog;

    // Phase 1: extract all spec modules.
    let mut spec_modules = BTreeMap::new();
    let mut source_definitions = extract_spec_modules(source_definitions, &mut spec_modules);
    let mut lib_definitions = extract_spec_modules(lib_definitions, &mut spec_modules);

    // Report errors for misplaced members
    for m in spec_modules.values() {
        let errors: Vec<_> = m
            .members
            .iter()
            .filter_map(|m| match m {
                ModuleMember::Function(f) => Some(vec![(
                    f.loc,
                    "functions not allowed in specification module".to_owned(),
                )]),
                ModuleMember::Struct(s) => Some(vec![(
                    s.loc,
                    "structs not allowed in specification module".to_owned(),
                )]),
                ModuleMember::Constant(c) => Some(vec![(
                    c.loc,
                    "constants not allowed in specification module".to_owned(),
                )]),
                ModuleMember::Use(_) | ModuleMember::Friend(_) | ModuleMember::Spec(_) => None,
            })
            .collect();
        if !errors.is_empty() {
            compilation_env.add_errors(errors);
        }
    }

    // Phase 2: Go over remaining proper modules and merge spec modules.
    merge_spec_modules(&mut source_definitions, &mut spec_modules);
    merge_spec_modules(&mut lib_definitions, &mut spec_modules);

    // Remaining spec modules could not be merged, report errors.
    for (_, m) in spec_modules {
        compilation_env.add_error(vec![(
            m.name.loc(),
            "Cannot associate specification with any target module in this compilation. \
                    A module specification cannot be compiled standalone."
                .to_owned(),
        )]);
    }
    Program {
        source_definitions,
        lib_definitions,
    }
}

fn extract_spec_modules(
    defs: Vec<Definition>,
    spec_modules: &mut BTreeMap<(Address, String), ModuleDefinition>,
) -> Vec<Definition> {
    use Definition::*;
    defs.into_iter()
        .filter_map(|def| match def {
            Module(m) => extract_spec_module(&None, m, spec_modules).map(Module),
            Address(attrs, loc, addr, module_defs) => {
                let addr_opt = Some(addr);
                let module_defs = module_defs
                    .into_iter()
                    .filter_map(|m| extract_spec_module(&addr_opt, m, spec_modules))
                    .collect::<Vec<_>>();
                Some(Address(attrs, loc, addr_opt.unwrap(), module_defs))
            }
            Definition::Script(s) => Some(Script(s)),
        })
        .collect()
}

fn extract_spec_module(
    address_opt: &Option<Address>,
    m: ModuleDefinition,
    spec_modules: &mut BTreeMap<(Address, String), ModuleDefinition>,
) -> Option<ModuleDefinition> {
    if m.is_spec_module {
        spec_modules.insert(module_key(address_opt, &m), m);
        None
    } else {
        Some(m)
    }
}

fn merge_spec_modules(
    defs: &mut Vec<Definition>,
    spec_modules: &mut BTreeMap<(Address, String), ModuleDefinition>,
) {
    use Definition::*;
    for def in defs.iter_mut() {
        match def {
            Module(m) => merge_spec_module(&None, m, spec_modules),
            Address(_attrs, _loc, addr, module_defs) => {
                let addr_opt = Some(*addr);
                for m in module_defs.iter_mut() {
                    merge_spec_module(&addr_opt, m, spec_modules)
                }
            }
            Script(..) => {}
        }
    }
}

fn merge_spec_module(
    address_opt: &Option<Address>,
    m: &mut ModuleDefinition,
    spec_modules: &mut BTreeMap<(Address, String), ModuleDefinition>,
) {
    if let Some(spec_module) = spec_modules.remove(&module_key(address_opt, m)) {
        let ModuleDefinition {
            attributes,
            members,
            ..
        } = spec_module;
        m.attributes.extend(attributes.into_iter());
        m.members.extend(members.into_iter());
    }
}

fn module_key(address_opt: &Option<Address>, def: &ModuleDefinition) -> (Address, String) {
    let name = def.name.0.value.clone();
    if let Some(sp!(_, a)) = def.address {
        (a, name)
    } else {
        // Having no context or defined address should not happen, but be robust about it
        // and always deliver an address here. An error will be reported elsewhere.
        (address_opt.clone().unwrap_or_else(Address::default), name)
    }
}
