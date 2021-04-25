// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use codespan_reporting::diagnostic::{Diagnostic, Label};
use itertools::Itertools;
#[allow(unused_imports)]
use log::warn;
use std::collections::BTreeSet;

use builder::module_builder::ModuleBuilder;
use move_binary_format::{
    access::ModuleAccess,
    file_format::{CompiledModule, FunctionDefinitionIndex, StructDefinitionIndex},
};
use move_core_types::account_address::AccountAddress;
use move_lang::{
    compiled_unit::{self, CompiledUnit},
    errors::Errors,
    expansion::ast::{ModuleDefinition, Program},
    move_continue_up_to, move_parse,
    parser::ast::ModuleIdent,
    shared::{unique_map::UniqueMap, Address, CompilationEnv, Flags},
    Pass as MovePass, PassResult as MovePassResult,
};
use num::{BigUint, Num};

use crate::{
    ast::{ModuleName, Spec},
    builder::model_builder::ModelBuilder,
    model::{FunId, FunctionData, GlobalEnv, Loc, ModuleData, ModuleId, StructId},
};

pub mod ast;
mod builder;
pub mod code_writer;
pub mod exp_generator;
pub mod exp_rewriter;
pub mod model;
pub mod native;
pub mod pragmas;
pub mod spec_translator;
pub mod symbol;
pub mod ty;

// =================================================================================================
// Entry Point

/// Build the move model with default compilation flags.
/// This collects transitive dependencies for move sources from the provided directory list.
pub fn run_model_builder(
    move_sources: &[String],
    deps_dir: &[String],
) -> anyhow::Result<GlobalEnv> {
    run_model_builder_with_compilation_flags(move_sources, deps_dir, Flags::empty())
}

/// Build the move model with supplied compilation flags.
/// This collects transitive dependencies for move sources from the provided directory list.
pub fn run_model_builder_with_compilation_flags(
    move_sources: &[String],
    deps_dir: &[String],
    flags: Flags,
) -> anyhow::Result<GlobalEnv> {
    let mut env = GlobalEnv::new();
    let mut compilation_env = CompilationEnv::new(flags.clone());

    // Step 1: parse the program to get comments and a separation of targets and dependencies.
    let (files, pprog_and_comments_res) =
        move_parse(&compilation_env, move_sources, deps_dir, None)?;
    let (comment_map, parsed_prog) = match pprog_and_comments_res {
        Err(errors) => {
            // Add source files so that the env knows how to translate locations of parse errors
            for fname in files.keys().sorted() {
                let fsrc = &files[fname];
                env.add_source(fname, fsrc, /* is_dep */ false);
            }
            add_move_lang_errors(&mut env, errors);
            return Ok(env);
        }
        Ok(res) => res,
    };
    // Add source files for targets and dependencies
    let dep_sources: BTreeSet<_> = parsed_prog
        .lib_definitions
        .iter()
        .map(|def| def.file())
        .collect();
    for fname in files.keys().sorted() {
        let fsrc = &files[fname];
        env.add_source(fname, fsrc, dep_sources.contains(fname));
    }

    // Add any documentation comments found by the Move compiler to the env.
    for (fname, documentation) in comment_map {
        let file_id = env.get_file_id(fname).expect("file name defined");
        env.add_documentation(file_id, documentation);
    }

    // Step 2: parse all sources in targets and dependencies, prepare for selective compilation.
    let all_sources: Vec<_> = files
        .into_iter()
        .map(|(fname, _)| fname.to_owned())
        .collect();
    // Run the compiler up to expansion
    let (_, pprog_and_comments_res) = move_parse(&compilation_env, &all_sources, &[], None)?;
    let (_, parsed_prog) = match pprog_and_comments_res {
        Err(errors) => {
            add_move_lang_errors(&mut env, errors);
            return Ok(env);
        }
        Ok(res) => res,
    };
    let expansion_ast = match move_continue_up_to(
        &mut compilation_env,
        None,
        MovePassResult::Parser(parsed_prog),
        MovePass::Expansion,
    ) {
        Err(errors) => {
            add_move_lang_errors(&mut env, errors);
            return Ok(env);
        }
        Ok(MovePassResult::Expansion(eprog)) => eprog,
        Ok(_) => unreachable!(),
    };
    // Extract the module/script closure
    let mut visited_modules = BTreeSet::new();
    let mut selective_files = BTreeSet::new();
    for (mident, mdef) in expansion_ast.modules.key_cloned_iter() {
        let src_file = mdef.loc.file();
        if !dep_sources.contains(src_file) {
            selective_files.insert(src_file.to_owned());
            collect_related_modules_recursive(mident, &expansion_ast.modules, &mut visited_modules);
        }
    }
    for (_, sdef) in expansion_ast.scripts {
        let src_file = sdef.loc.file();
        if !dep_sources.contains(src_file) {
            selective_files.insert(src_file.to_owned());
            for neighbor in sdef.immediate_neighbors {
                collect_related_modules_recursive(
                    neighbor.into_module_ident(),
                    &expansion_ast.modules,
                    &mut visited_modules,
                );
            }
        }
    }
    for mident in visited_modules {
        selective_files.insert(
            expansion_ast
                .modules
                .get(&mident)
                .unwrap()
                .loc
                .file()
                .to_owned(),
        );
    }

    // Step 3: selective compilation.
    let selective_sources: Vec<_> = selective_files.into_iter().collect();

    // Run the compiler up to expansion and clone a copy of the expansion program ast
    let mut compilation_env = CompilationEnv::new(flags);
    let (_, pprog_and_comments_res) = move_parse(&compilation_env, &selective_sources, &[], None)?;
    let (_, parsed_prog) = match pprog_and_comments_res {
        Err(errors) => {
            add_move_lang_errors(&mut env, errors);
            return Ok(env);
        }
        Ok(res) => res,
    };
    let (expansion_ast, expansion_result) = match move_continue_up_to(
        &mut compilation_env,
        None,
        MovePassResult::Parser(parsed_prog),
        MovePass::Expansion,
    ) {
        Err(errors) => {
            add_move_lang_errors(&mut env, errors);
            return Ok(env);
        }
        Ok(MovePassResult::Expansion(eprog)) => (eprog.clone(), MovePassResult::Expansion(eprog)),
        Ok(_) => unreachable!(),
    };
    // Run the compiler fully to the compiled units
    let units = match move_continue_up_to(
        &mut compilation_env,
        None,
        expansion_result,
        MovePass::Compilation,
    ) {
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
    run_spec_checker(&mut env, verified_units, expansion_ast);
    Ok(env)
}

fn collect_related_modules_recursive(
    mident: ModuleIdent,
    modules: &UniqueMap<ModuleIdent, ModuleDefinition>,
    visited: &mut BTreeSet<ModuleIdent>,
) {
    if visited.contains(&mident) {
        return;
    }
    let mdef = modules.get(&mident).unwrap();
    let deps: BTreeSet<_> = mdef
        .immediate_neighbors
        .iter()
        .map(|neighbor| neighbor.clone().into_module_ident())
        .collect();
    visited.insert(mident);
    for next_mident in deps {
        collect_related_modules_recursive(next_mident, modules, visited);
    }
}

/// Build a `GlobalEnv` from a collection of `CompiledModule`'s. The `modules` list must be
/// topologically sorted by the dependency relation (i.e., a child node in the dependency graph
/// should appear earlier in the vector than its parents).
pub fn run_bytecode_model_builder<'a>(
    modules: impl IntoIterator<Item = &'a CompiledModule>,
) -> anyhow::Result<GlobalEnv> {
    let mut env = GlobalEnv::new();
    for (i, m) in modules.into_iter().enumerate() {
        let id = m.self_id();
        let addr = addr_to_big_uint(id.address());
        let module_name = ModuleName::new(addr, env.symbol_pool().make(id.name().as_str()));
        let module_id = ModuleId::new(i);
        let mut module_data = ModuleData::stub(module_name.clone(), module_id, m.clone());

        // add functions
        for (i, def) in m.function_defs().iter().enumerate() {
            let def_idx = FunctionDefinitionIndex(i as u16);
            let name = m.identifier_at(m.function_handle_at(def.function).name);
            let symbol = env.symbol_pool().make(name.as_str());
            let fun_id = FunId::new(symbol);
            let data = FunctionData::stub(symbol, def_idx, def.function);
            module_data.function_data.insert(fun_id, data);
            module_data.function_idx_to_id.insert(def_idx, fun_id);
        }

        // add structs
        for (i, def) in m.struct_defs().iter().enumerate() {
            let def_idx = StructDefinitionIndex(i as u16);
            let name = m.identifier_at(m.struct_handle_at(def.struct_handle).name);
            let symbol = env.symbol_pool().make(name.as_str());
            let struct_id = StructId::new(symbol);
            let data = env.create_struct_data(&m, def_idx, symbol, Loc::default(), Spec::default());
            module_data.struct_data.insert(struct_id, data);
            module_data.struct_idx_to_id.insert(def_idx, struct_id);
        }

        env.module_data.push(module_data);
    }
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
fn run_spec_checker(env: &mut GlobalEnv, units: Vec<CompiledUnit>, mut eprog: Program) {
    let mut builder = ModelBuilder::new(env);
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
                        attributes,
                        loc,
                        immediate_neighbors,
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
                    let ident = ModuleIdent {
                        locs: (loc, loc),
                        value: (Address::default(), function_name.0.value.clone()),
                    };
                    let mut function_infos = UniqueMap::new();
                    function_infos
                        .add(function_name.clone(), function_info)
                        .unwrap();
                    // Construct a pseudo module definition.
                    let mut functions = UniqueMap::new();
                    functions.add(function_name, function).unwrap();
                    let expanded_module = ModuleDefinition {
                        attributes,
                        loc,
                        dependency_order: usize::MAX,
                        immediate_neighbors,
                        is_source_module: true,
                        friends: UniqueMap::new(),
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
        let loc = builder.to_loc(&expanded_module.loc);
        let module_name = ModuleName::from_str(
            &module_id.value.0.to_string(),
            builder.env.symbol_pool().make(&module_id.value.1),
        );
        let module_id = ModuleId::new(module_count);
        let mut module_translator = ModuleBuilder::new(&mut builder, module_id, module_name);
        module_translator.translate(
            loc,
            expanded_module,
            compiled_module,
            source_map,
            function_infos,
        );
    }
    // After all specs have been processed, warn about any unused schemas.
    builder.warn_unused_schemas();
}

// =================================================================================================
// Helpers

/// Converts an address identifier to a number representing the address.
pub fn addr_to_big_uint(addr: &AccountAddress) -> BigUint {
    BigUint::from_str_radix(&addr.to_string(), 16).unwrap()
}

/// Converts a biguint into an account address
pub fn big_uint_to_addr(i: &BigUint) -> AccountAddress {
    // TODO: do this in more efficient way (e.g., i.to_le_bytes() and pad out the resulting Vec<u8>
    // to ADDRESS_LENGTH
    AccountAddress::from_hex_literal(&format!("{:#x}", i)).unwrap()
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
