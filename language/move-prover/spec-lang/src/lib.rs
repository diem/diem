// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use crate::{
    ast::ModuleName,
    env::{GlobalEnv, ModuleId},
    translate::{ModuleTranslator, Translator},
};
use anyhow::anyhow;
use codespan_reporting::diagnostic::{Diagnostic, Label};
use itertools::Itertools;
use move_lang::{
    errors::Errors, expansion::ast::Program, move_compile_no_report,
    move_compile_to_expansion_no_report, shared::Address, to_bytecode::translate::CompiledUnit,
};

pub mod ast;
pub mod env;
pub mod symbol;
mod translate;
pub mod ty;

#[allow(unused_imports)]
use log::{info, warn};

// =================================================================================================
// Entry Point

pub fn run_spec_lang_compiler(
    targets: Vec<String>,
    deps: Vec<String>,
    address_opt: Option<&str>,
) -> anyhow::Result<GlobalEnv> {
    let address_opt = address_opt
        .map(Address::parse_str)
        .transpose()
        .map_err(|s| anyhow!(s))?;

    let mut env = GlobalEnv::new();
    // First pass: compile move code.
    let (files, units_or_errors) = move_compile_no_report(&targets, &deps, address_opt)?;
    for (fname, fsrc) in files {
        env.add_source(fname, &fsrc);
    }
    match units_or_errors {
        Err(errors) => {
            add_move_lang_errors(&mut env, errors);
        }
        Ok(units) => {
            let (verified_units, errors) = move_lang::to_bytecode::translate::verify_units(units);
            if !errors.is_empty() {
                add_move_lang_errors(&mut env, errors);
            } else {
                // Now compile again, up to expansion phase, to get hand on the expansion AST
                // which we merge with the verified units. This time we expect no errors.
                // The alternative to do a second parse and expansion pass is to make the expansion
                // AST clonable and tee it somehow out of the regular compile chain.
                let (_, eprog_or_errors) =
                    move_compile_to_expansion_no_report(&targets, &deps, address_opt)?;
                let eprog = eprog_or_errors.expect("no compilation errors");
                // Run the spec checker on verified units plus expanded AST. This will
                // populate the environment including any errors.
                run_spec_checker(&mut env, verified_units, eprog)?;
            }
        }
    };
    Ok(env)
}

fn add_move_lang_errors(env: &mut GlobalEnv, errors: Errors) {
    let mk_label = |env: &mut GlobalEnv, err: (move_ir_types::location::Loc, String)| {
        let loc = env.to_loc(&err.0);
        Label::new(loc.file_id(), loc.span(), err.1)
    };
    for mut error in errors {
        let primary = error.remove(0);
        let diag = Diagnostic::new_error("", mk_label(env, primary))
            .with_secondary_labels(error.into_iter().map(|e| mk_label(env, e)));
        env.add_diag(diag);
    }
}

fn run_spec_checker(
    env: &mut GlobalEnv,
    units: Vec<CompiledUnit>,
    mut eprog: Program,
) -> anyhow::Result<()> {
    let mut translator = Translator::new(env);
    // Merge the compiled units with the expanded program, preserving the order of the compiled
    // units which is topological w.r.t. use relation.
    let modules = units
        .into_iter()
        .filter_map(|u| {
            if let CompiledUnit::Module(name, compiled_module, source_map) = u {
                let expanded_id = eprog
                    .modules
                    .iter()
                    .map(|(module_id, _)| module_id)
                    .find(|module_id| module_id.0.value.name.0.value == name.0.value);
                if let Some(module_id) = expanded_id {
                    let expanded_module = eprog.modules.remove(&module_id).unwrap();
                    Some((module_id, expanded_module, compiled_module, source_map))
                } else {
                    warn!(
                        "[internal] cannot associate bytecode module `{}` with AST",
                        name.0.value
                    );
                    None
                }
            } else {
                None
            }
        })
        .collect_vec();
    for (module_count, (module_id, expanded_module, compiled_module, source_map)) in
        modules.into_iter().enumerate()
    {
        let loc = translator.to_loc(&expanded_module.loc);
        let module_name = ModuleName::from_str(
            &module_id.0.value.address.to_string(),
            translator
                .env
                .symbol_pool()
                .make(&module_id.0.value.name.0.value),
        );
        let module_id = ModuleId::new(module_count);
        let mut module_translator = ModuleTranslator::new(&mut translator, module_id, module_name);
        module_translator.translate(loc, expanded_module, compiled_module, Some(source_map));
    }
    Ok(())
}

// =================================================================================================
// Crate Helpers

/// Helper to project the 1st element from a vector of pairs.
pub(crate) fn project_1st<T: Clone, R>(v: &[(T, R)]) -> Vec<T> {
    v.iter().map(|(x, _)| x.clone()).collect()
}

/// Helper to project the 2nd element from a vector of pairs.
pub(crate) fn project_2nd<T, R: Clone>(v: &[(T, R)]) -> Vec<R> {
    v.iter().map(|(_, x)| x.clone()).collect()
}
