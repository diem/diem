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
use move_lang::{
    compiled_unit::{self, CompiledUnit},
    errors::Errors,
    expansion::ast::Program,
    move_compile_no_report, move_compile_to_expansion_no_report,
    parser::ast::FunctionName,
    shared::Address,
};

pub mod ast;
pub mod env;
pub mod symbol;
mod translate;
pub mod ty;

use crate::env::SCRIPT_MODULE_NAME;
#[allow(unused_imports)]
use log::{info, warn};
use move_ir_types::location::Spanned;
use move_lang::{
    expansion::ast::ModuleDefinition,
    parser::ast::{ModuleIdent, ModuleIdent_},
    shared::{unique_map::UniqueMap, Name},
};

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

    // Construct all sources from targets and deps, as we need bytecode for all of them.
    let mut all_sources = targets;
    all_sources.extend(deps.clone());
    let mut env = GlobalEnv::new();
    // First pass: compile move code.
    let (files, units_or_errors) = move_compile_no_report(&all_sources, &[], address_opt)?;
    for (fname, fsrc) in files {
        env.add_source(fname, &fsrc, deps.contains(&fname.to_string()));
    }
    match units_or_errors {
        Err(errors) => {
            add_move_lang_errors(&mut env, errors);
        }
        Ok(units) => {
            let (verified_units, errors) = compiled_unit::verify_units(units);
            if !errors.is_empty() {
                add_move_lang_errors(&mut env, errors);
            } else {
                // Now compile again, up to expansion phase, to get hand on the expansion AST
                // which we merge with the verified units. This time we expect no errors.
                // The alternative to do a second parse and expansion pass is to make the expansion
                // AST clonable and tee it somehow out of the regular compile chain.
                let (_, eprog_or_errors) =
                    move_compile_to_expansion_no_report(&all_sources, &[], address_opt)?;
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

#[allow(deprecated)]
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
        .flat_map(|unit| {
            Some(match unit {
                CompiledUnit::Module {
                    ident,
                    module,
                    source_map,
                    function_infos,
                } => {
                    let expanded_module = match eprog.modules.remove(&ident) {
                        Some(m) => m,
                        None => {
                            warn!(
                                "[internal] cannot associate bytecode module `{}` with AST",
                                ident
                            );
                            return None;
                        }
                    };
                    (ident, expanded_module, module, source_map, function_infos)
                }
                CompiledUnit::Script {
                    loc: _loc,
                    key,
                    script,
                    source_map,
                    function_info,
                } => {
                    let move_lang::expansion::ast::Script {
                        loc,
                        uses,
                        unused_aliases: _unused_aliases,
                        function_name,
                        function,
                        specs,
                    } = match eprog.scripts.remove(&key) {
                        Some(s) => s,
                        None => {
                            warn!(
                                "[internal] cannot associate bytecode script `{}` with AST",
                                key
                            );
                            return None;
                        }
                    };
                    // Convert the script into a module.
                    let ident = ModuleIdent(Spanned {
                        loc,
                        value: ModuleIdent_ {
                            name: move_lang::parser::ast::ModuleName(Name {
                                loc,
                                value: SCRIPT_MODULE_NAME.to_string(),
                            }),
                            address: Address::default(),
                        },
                    });
                    let mut function_infos = UniqueMap::new();
                    function_infos
                        .add(FunctionName(Name { loc, value: key }), function_info)
                        .unwrap();
                    // Construct a pseudo module definition.
                    let mut functions = UniqueMap::new();
                    functions.add(function_name.clone(), function).unwrap();
                    // As we now know the real function name and address, replace it in the
                    // data we got from bytecode.
                    let function_info = function_infos.into_iter().next().unwrap().1;
                    function_infos = UniqueMap::new();
                    function_infos.add(function_name, function_info).unwrap();
                    let expanded_module = ModuleDefinition {
                        loc,
                        uses,
                        unused_aliases: vec![],
                        is_source_module: true,
                        structs: UniqueMap::new(),
                        functions,
                        specs,
                    };
                    let module = script.into_module().1;
                    (ident, expanded_module, module, source_map, function_infos)
                }
            })
        })
        .enumerate();
    for (module_count, (module_id, expanded_module, compiled_module, source_map, function_infos)) in
        modules
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
        module_translator.translate(
            loc,
            expanded_module,
            compiled_module,
            source_map,
            function_infos,
        );
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
