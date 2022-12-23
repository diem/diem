// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    parser::ast::{Definition, Program},
    shared::*,
};
use std::collections::BTreeSet;

use super::ast::LeadingNameAccess_;

/// Given a parsed program, if a module id is found in both the source and lib definitions, filter
/// out the lib definition and re-construct a new parsed program
pub fn program(compilation_env: &CompilationEnv, prog: Program) -> Program {
    if !compilation_env.flags().sources_shadow_deps() {
        return prog;
    }

    let Program {
        source_definitions,
        lib_definitions,
    } = prog;
    let mut modules_defined_in_src = BTreeSet::new();
    for def in &source_definitions {
        match def {
            Definition::Address(a) => {
                let addr = a.addr.value;
                for module in &a.modules {
                    modules_defined_in_src.insert((addr, module.name));
                }
            }
            Definition::Module(module) => {
                modules_defined_in_src.insert((
                    module.address.map(|a| a.value).unwrap_or_else(|| {
                        LeadingNameAccess_::AnonymousAddress(AddressBytes::DEFAULT_ERROR_BYTES)
                    }),
                    module.name,
                ));
            }
            Definition::Script(_) => (),
        }
    }

    // Shadow (i.e., filter out) library definitions with the following criteria
    //  1. this library definition is an Address space AST and any module in the address space is
    //    already defined in the source.
    //  2. this library definition is a Module AST and the module is already defined in the source.
    let lib_definitions = lib_definitions
        .into_iter()
        .map(|def| match def {
            Definition::Address(mut a) => {
                let addr = a.addr.value;
                let modules = std::mem::take(&mut a.modules);
                a.modules = modules
                    .into_iter()
                    .filter(|module| !modules_defined_in_src.contains(&(addr, module.name)))
                    .collect();
                Definition::Address(a)
            }
            def => def,
        })
        .filter(|def| match def {
            Definition::Address(_) => true,
            Definition::Module(module) => !modules_defined_in_src.contains(&(
                module.address.map(|a| a.value).unwrap_or_else(|| {
                    LeadingNameAccess_::AnonymousAddress(AddressBytes::DEFAULT_ERROR_BYTES)
                }),
                module.name,
            )),
            Definition::Script(_) => false,
        })
        .collect();

    Program {
        source_definitions,
        lib_definitions,
    }
}
