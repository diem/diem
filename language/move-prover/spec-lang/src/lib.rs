// Copyright (c) The Diem Core Contributors
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
    move_continue_up_to, move_parse,
    shared::Address,
    Pass as MovePass, PassResult as MovePassResult,
};

pub mod ast;
pub mod code_writer;
pub mod env;
pub mod symbol;
mod translate;
pub mod ty;

use itertools::Itertools;
#[allow(unused_imports)]
use log::warn;
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
    // Parse the program
    let (files, pprog_and_comments_res) = move_parse(&all_sources, &[], address_opt, None)?;
    for fname in files.keys().sorted() {
        let fsrc = &files[fname];
        env.add_source(fname, fsrc, deps.contains(&fname.to_string()));
    }
    // Add any documentation comments found by the Move compiler to the env.
    let (comment_map, addr_opt, parsed_prog) = match pprog_and_comments_res {
        Err(errors) => {
            add_move_lang_errors(&mut env, errors);
            return Ok(env);
        }
        Ok(res) => res,
    };
    for (fname, documentation) in comment_map {
        let file_id = env.get_file_id(fname).expect("file name defined");
        env.add_documentation(file_id, documentation);
    }
    // Run the compiler up to expansion and clone a copy of the expansion program ast
    let (expansion_ast, expansion_result) = match move_continue_up_to(
        MovePassResult::Parser(addr_opt, parsed_prog),
        MovePass::Expansion,
    ) {
        Err(errors) => {
            add_move_lang_errors(&mut env, errors);
            return Ok(env);
        }
        Ok(MovePassResult::Expansion(eprog, eerrors)) => {
            (eprog.clone(), MovePassResult::Expansion(eprog, eerrors))
        }
        Ok(_) => unreachable!(),
    };
    // Run the compiler fully to the compiled units
    let units = match move_continue_up_to(expansion_result, MovePass::Compilation) {
        Err(errors) => {
            add_move_lang_errors(&mut env, errors);
            return Ok(env);
        }
        Ok(MovePassResult::Compilation(units)) => units,
        Ok(_) => unreachable!(),
    };
    // Check for bytecode verifier errors (there should not be any)
    let (verified_units, errors) = compiled_unit::verify_units(units);
    if !errors.is_empty() {
        add_move_lang_errors(&mut env, errors);
        return Ok(env);
    }

    // Now that it is known that the program has no errors, run the spec checker on verified units
    // plus expanded AST. This will populate the environment including any errors.
    run_spec_checker(&mut env, verified_units, expansion_ast)?;
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
                        function_name,
                        constants,
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
                                value: function_name.0.value.clone(),
                            }),
                            address: Address::default(),
                        },
                    });
                    let mut function_infos = UniqueMap::new();
                    function_infos
                        .add(function_name.clone(), function_info)
                        .unwrap();
                    // Construct a pseudo module definition.
                    let mut functions = UniqueMap::new();
                    functions.add(function_name, function).unwrap();
                    let expanded_module = ModuleDefinition {
                        loc,
                        is_source_module: true,
                        structs: UniqueMap::new(),
                        constants,
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
