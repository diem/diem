// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use codespan_reporting::diagnostic::{Diagnostic, Label};
use itertools::Itertools;
#[allow(unused_imports)]
use log::warn;
use std::{
    collections::BTreeSet,
    fs::{self, File},
    path::Path,
};

use builder::module_builder::ModuleBuilder;
use move_lang::{
    compiled_unit::{self, CompiledUnit},
    errors::{Errors, FilesSourceText},
    expansion::ast::{ModuleDefinition, Program},
    move_continue_up_to, move_parse,
    naming::ast::Program as IProgram,
    output_compiled_units,
    parser::ast::ModuleIdent,
    shared::{ast_debug, unique_map::UniqueMap, Address, CompilationEnv, Flags},
    Pass as MovePass, PassResult as MovePassResult,
};
use vm::{
    access::ModuleAccess,
    file_format::{CompiledModule, FunctionDefinitionIndex, StructDefinitionIndex},
};

use crate::{
    ast::{ModuleName, Spec},
    builder::model_builder::ModelBuilder,
    model::{FunId, FunctionData, GlobalEnv, Loc, ModuleData, ModuleEnv, ModuleId, StructId},
};

pub mod ast;
mod builder;
pub mod code_writer;
pub mod exp_generator;
pub mod exp_rewriter;
mod instrumenter;
pub mod model;
pub mod pragmas;
pub mod spec_translator;
pub mod symbol;
pub mod ty;

// =================================================================================================
// Entry Point

/// Build the move model. This collects transitive dependencies for move sources
/// from the provided directory list.
pub fn run_model_builder(
    move_sources: &[String],
    deps_dir: &[String],
) -> anyhow::Result<GlobalEnv> {
    let (mut env, compilation_result) = compile_program(
        move_sources,
        deps_dir,
        /* save_ast_for_instrument */ false,
    )?;
    match compilation_result {
        None => Ok(env),
        Some((verified_units, expansion_ast, _, _)) => {
            // Run the spec checker on verified units plus expanded AST.
            // This will populate the environment including any errors.
            run_spec_checker(&mut env, verified_units, expansion_ast);
            Ok(env)
        }
    }
}

pub fn run_spec_instrumenter(
    move_sources: &[String],
    deps_dir: &[String],
    output_dir: &str,
    dump_asts: bool,
) -> anyhow::Result<GlobalEnv> {
    const FILE_OLD_AST: &str = "program.ast.old";
    const FILE_NEW_AST: &str = "program.ast.new";

    // Compile the program and make sure that specs are built without error
    let (mut env, compilation_result) = compile_program(
        move_sources,
        deps_dir,
        /* save_ast_for_instrument */ true,
    )?;
    if compilation_result.is_none() {
        // it is probably a bad idea to continue spec instrumentation if the programs by themselves
        // are already failing compilation
        return Ok(env);
    }

    let (verified_units, expansion_ast, hlir_ast_opt, source_files) = compilation_result.unwrap();
    // Run the spec checker on verified units plus expanded AST.
    // This will populate the environment including any errors.
    run_spec_checker(&mut env, verified_units.clone(), expansion_ast);
    if env.has_errors() {
        // do not instrument specs if there are any errors in the spec population phase
        return Ok(env);
    }
    let hlir_ast = hlir_ast_opt.unwrap();

    // Prepare output directory
    let output_base = Path::new(output_dir);
    fs::create_dir_all(output_base)?;

    // Dump the old program AST if requested
    if dump_asts {
        let mut file = File::create(output_base.join(FILE_OLD_AST))?;
        ast_debug::print_to_file(&hlir_ast, true, &mut file)?;
    }

    // Entry point to the instrumentation logic
    let instrumented_ast = instrumenter::translate::run(&env, &verified_units, hlir_ast);
    if env.has_errors() {
        // do not compile the new program if there are any errors in the instrumentation phase
        return Ok(env);
    }

    // Dump the new program AST if requested
    if dump_asts {
        let mut file = File::create(output_base.join(FILE_NEW_AST))?;
        ast_debug::print_to_file(&instrumented_ast, true, &mut file)?;
    }

    // Run the compiler fully on the instrumented program
    let mut compilation_env = CompilationEnv::new(Flags::empty());
    let units = match move_continue_up_to(
        &mut compilation_env,
        None,
        MovePassResult::Naming(instrumented_ast),
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

    // Output the instrumented compilation units
    output_compiled_units(false, source_files, verified_units, output_dir)?;

    // Now we know that the compiled units have successfully compiled and passed the verification
    Ok(env)
}

/// Build a `GlobalEnv` from a collection of `CompiledModule`'s. The `modules` list must be
/// topologically sorted by the dependency relation (i.e., a child node in the dependency graph
/// should appear earlier in the vector than its parents).
pub fn run_bytecode_model_builder(modules: Vec<CompiledModule>) -> anyhow::Result<GlobalEnv> {
    let mut env = GlobalEnv::new();
    for (i, m) in modules.into_iter().enumerate() {
        let id = m.self_id();
        let addr = ModuleEnv::addr_to_big_uint(id.address());
        let module_name = ModuleName::new(addr, env.symbol_pool().make(id.name().as_str()));
        let module_id = ModuleId::new(i);
        let mut module_data = ModuleData::stub(module_name.clone(), module_id, m.clone());

        // add functions
        for (def_idx, def) in m.function_defs().iter().enumerate() {
            let name = m.identifier_at(m.function_handle_at(def.function).name);
            let symbol = env.symbol_pool().make(name.as_str());
            let data = FunctionData::stub(
                symbol,
                FunctionDefinitionIndex(def_idx as u16),
                def.function,
            );
            module_data.function_data.insert(FunId::new(symbol), data);
        }

        // add structs
        for (def_idx, def) in m.struct_defs().iter().enumerate() {
            let name = m.identifier_at(m.struct_handle_at(def.struct_handle).name);
            let symbol = env.symbol_pool().make(name.as_str());
            let data = env.create_struct_data(
                &m,
                StructDefinitionIndex(def_idx as u16),
                symbol,
                Loc::default(),
                Spec::default(),
            );
            module_data.struct_data.insert(StructId::new(symbol), data);
        }

        env.module_data.push(module_data);
    }
    Ok(env)
}

fn compile_program(
    move_sources: &[String],
    deps_dir: &[String],
    save_ast_for_instrument: bool,
) -> anyhow::Result<(
    GlobalEnv,
    Option<(
        Vec<CompiledUnit>,
        Program,
        Option<IProgram>,
        FilesSourceText,
    )>,
)> {
    let mut env = GlobalEnv::new();

    // Step 1: parse the program to get comments and a separation of targets and dependencies.
    let (files, pprog_and_comments_res) = move_parse(
        move_sources,
        deps_dir,
        None,
        /* sources_shadow_deps */ true,
    )?;
    let (comment_map, parsed_prog) = match pprog_and_comments_res {
        Err(errors) => {
            // Add source files so that the env knows how to translate locations of parse errors
            for fname in files.keys().sorted() {
                let fsrc = &files[fname];
                env.add_source(fname, fsrc, /* is_dep */ false);
            }
            add_move_lang_errors(&mut env, errors);
            return Ok((env, None));
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

    // Step 2: compile all sources in targets and dependencies, as we need bytecode for all of them.
    let all_sources: Vec<_> = files
        .into_iter()
        .map(|(fname, _)| fname.to_owned())
        .collect();
    // Run the compiler up to expansion and clone a copy of the expansion program ast
    let (files, pprog_and_comments_res) = move_parse(&all_sources, &[], None, false)?;
    let (_, parsed_prog) = match pprog_and_comments_res {
        Err(errors) => {
            add_move_lang_errors(&mut env, errors);
            return Ok((env, None));
        }
        Ok(res) => res,
    };
    // Run the compiler up to expansion and clone a copy of the expansion program ast
    let mut compilation_env = CompilationEnv::new(Flags::empty());
    let (expansion_ast, expansion_result) = match move_continue_up_to(
        &mut compilation_env,
        None,
        MovePassResult::Parser(parsed_prog),
        MovePass::Expansion,
    ) {
        Err(errors) => {
            add_move_lang_errors(&mut env, errors);
            return Ok((env, None));
        }
        Ok(MovePassResult::Expansion(eprog)) => (eprog.clone(), MovePassResult::Expansion(eprog)),
        Ok(_) => unreachable!(),
    };
    // Run the compiler up to naming and clone a copy of the program ast, if requested
    let (ast_for_instrument_opt, intermediate_pass_result) = match move_continue_up_to(
        &mut compilation_env,
        None,
        expansion_result,
        MovePass::Naming,
    ) {
        Err(errors) => {
            add_move_lang_errors(&mut env, errors);
            return Ok((env, None));
        }
        Ok(MovePassResult::Naming(iprog)) => {
            let ast_opt = if save_ast_for_instrument {
                Some(iprog.clone())
            } else {
                None
            };
            (ast_opt, MovePassResult::Naming(iprog))
        }
        Ok(_) => unreachable!(),
    };
    // Run the compiler fully to the compiled units
    let units = match move_continue_up_to(
        &mut compilation_env,
        None,
        intermediate_pass_result,
        MovePass::Compilation,
    ) {
        Err(errors) => {
            add_move_lang_errors(&mut env, errors);
            return Ok((env, None));
        }
        Ok(MovePassResult::Compilation(units)) => units,
        Ok(_) => unreachable!(),
    };
    // Check for bytecode verifier errors (there should not be any)
    let (verified_units, errors) = compiled_unit::verify_units(units);
    if !errors.is_empty() {
        add_move_lang_errors(&mut env, errors);
        return Ok((env, None));
    }
    // Now that it is known that the program has no errors
    Ok((
        env,
        Some((verified_units, expansion_ast, ast_for_instrument_opt, files)),
    ))
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
// Crate Helpers

/// Helper to project the 1st element from a vector of pairs.
pub(crate) fn project_1st<T: Clone, R>(v: &[(T, R)]) -> Vec<T> {
    v.iter().map(|(x, _)| x.clone()).collect()
}

/// Helper to project the 2nd element from a vector of pairs.
pub(crate) fn project_2nd<T, R: Clone>(v: &[(T, R)]) -> Vec<R> {
    v.iter().map(|(_, x)| x.clone()).collect()
}
