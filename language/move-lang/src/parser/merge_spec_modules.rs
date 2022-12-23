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
    diag,
    parser::ast::{Definition, LeadingNameAccess_, ModuleDefinition, ModuleMember, Program},
    shared::*,
};
use move_symbol_pool::Symbol;
use std::collections::BTreeMap;

/// Given a parsed program, merge all specification modules into their target modules.
pub fn program(compilation_env: &mut CompilationEnv, prog: Program) -> Program {
    let Program {
        source_definitions,
        lib_definitions,
    } = prog;

    // Phase 1: extract all spec modules.
    let mut spec_modules = BTreeMap::new();
    let mut source_definitions = extract_spec_modules(&mut spec_modules, source_definitions);
    let mut lib_definitions = extract_spec_modules(&mut spec_modules, lib_definitions);

    // Report errors for misplaced members
    for m in spec_modules.values() {
        for mem in &m.members {
            let (loc, msg) = match mem {
                ModuleMember::Function(f) => {
                    (f.loc, "functions not allowed in specification module")
                }
                ModuleMember::Struct(s) => (s.loc, "structs not allowed in specification module"),
                ModuleMember::Constant(c) => {
                    (c.loc, "constants not allowed in specification module")
                }
                ModuleMember::Use(_) | ModuleMember::Friend(_) | ModuleMember::Spec(_) => continue,
            };
            compilation_env.add_diag(diag!(Declarations::InvalidSpec, (loc, msg)))
        }
    }

    // Phase 2: Go over remaining proper modules and merge spec modules.
    merge_spec_modules(&mut spec_modules, &mut source_definitions);
    merge_spec_modules(&mut spec_modules, &mut lib_definitions);

    // Remaining spec modules could not be merged, report errors.
    for (_, m) in spec_modules {
        let msg = "Cannot associate specification with any target module in this compilation. A \
                   module specification cannot be compiled standalone.";
        compilation_env.add_diag(diag!(Declarations::InvalidSpec, (m.name.loc(), msg)))
    }
    Program {
        source_definitions,
        lib_definitions,
    }
}

fn extract_spec_modules(
    spec_modules: &mut BTreeMap<(Option<LeadingNameAccess_>, Symbol), ModuleDefinition>,
    defs: Vec<Definition>,
) -> Vec<Definition> {
    use Definition::*;
    defs.into_iter()
        .filter_map(|def| match def {
            Module(m) => extract_spec_module(spec_modules, None, m).map(Module),
            Address(mut a) => {
                let addr_ = Some(&a.addr.value);
                a.modules = a
                    .modules
                    .into_iter()
                    .filter_map(|m| extract_spec_module(spec_modules, addr_, m))
                    .collect::<Vec<_>>();
                Some(Address(a))
            }
            Definition::Script(s) => Some(Script(s)),
        })
        .collect()
}

fn extract_spec_module(
    spec_modules: &mut BTreeMap<(Option<LeadingNameAccess_>, Symbol), ModuleDefinition>,
    address_opt: Option<&LeadingNameAccess_>,
    m: ModuleDefinition,
) -> Option<ModuleDefinition> {
    if m.is_spec_module {
        spec_modules.insert(module_key(address_opt, &m), m);
        None
    } else {
        Some(m)
    }
}

fn merge_spec_modules(
    spec_modules: &mut BTreeMap<(Option<LeadingNameAccess_>, Symbol), ModuleDefinition>,
    defs: &mut Vec<Definition>,
) {
    use Definition::*;
    for def in defs.iter_mut() {
        match def {
            Module(m) => merge_spec_module(spec_modules, None, m),
            Address(a) => {
                let addr_ = Some(&a.addr.value);
                for m in &mut a.modules {
                    merge_spec_module(spec_modules, addr_, m)
                }
            }
            Script(_) => {}
        }
    }
}

fn merge_spec_module(
    spec_modules: &mut BTreeMap<(Option<LeadingNameAccess_>, Symbol), ModuleDefinition>,
    address_opt: Option<&LeadingNameAccess_>,
    m: &mut ModuleDefinition,
) {
    if let Some(spec_module) = spec_modules.remove(&module_key(address_opt, m)) {
        let ModuleDefinition {
            attributes,
            members,
            loc: _,
            address: _,
            name: _,
            is_spec_module,
        } = spec_module;
        assert!(is_spec_module);
        m.attributes.extend(attributes.into_iter());
        m.members.extend(members.into_iter());
    }
}

fn module_key(
    address_opt: Option<&LeadingNameAccess_>,
    m: &ModuleDefinition,
) -> (Option<LeadingNameAccess_>, Symbol) {
    let addr_ = match &m.address {
        Some(sp!(_, a_)) => Some(*a_),
        None => address_opt.copied(),
    };
    (addr_, m.name.value())
}
