// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use codespan_reporting::diagnostic::{Diagnostic, Label};
use itertools::Itertools;
#[allow(unused_imports)]
use log::warn;
use std::{fs, path::Path};

use builder::module_builder::ModuleBuilder;
use bytecode_source_map::mapping::SourceMapping;
use disassembler::disassembler::{Disassembler, DisassemblerOptions};
use move_lang::{
    compiled_unit::{self, CompiledUnit},
    errors::{Errors, FilesSourceText},
    expansion::ast::{ModuleDefinition, Program},
    move_continue_up_to, move_parse,
    parser::ast::ModuleIdent,
    shared::{unique_map::UniqueMap, Address},
    Pass as MovePass, PassResult as MovePassResult, MOVE_COMPILED_EXTENSION,
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
pub mod exp_rewriter;
pub mod model;
pub mod pragmas;
pub mod symbol;
pub mod ty;

// =================================================================================================
// Entry Point

pub fn run_model_builder(
    target_sources: Vec<String>,
    other_sources: Vec<String>,
) -> anyhow::Result<GlobalEnv> {
    let (mut env, compilation_result) = compile_program(target_sources, other_sources)?;
    match compilation_result {
        None => Ok(env),
        Some((verified_units, expansion_ast, _)) => {
            // Run the spec checker on verified units plus expanded AST.
            // This will populate the environment including any errors.
            run_spec_checker(&mut env, verified_units, expansion_ast);
            Ok(env)
        }
    }
}

pub fn run_spec_instrumenter(
    target_sources: Vec<String>,
    other_sources: Vec<String>,
    output_dir: &str,
    dump_bytecode: bool,
) -> anyhow::Result<GlobalEnv> {
    const SCRIPT_SUB_DIR: &str = "scripts";
    const MODULE_SUB_DIR: &str = "modules";
    const MOVE_DISASSEMBLY_EXTENSION: &str = "disas";

    let (mut env, compilation_result) = compile_program(target_sources, other_sources)?;
    if compilation_result.is_none() {
        // it is probably a bad idea to continue spec instrumentation if the programs by themselves
        // are already failing compilation
        return Ok(env);
    }

    let (verified_units, expansion_ast, _) = compilation_result.unwrap();
    // Run the spec checker on verified units plus expanded AST.
    // This will populate the environment including any errors.
    run_spec_checker(&mut env, verified_units, expansion_ast.clone());
    if env.has_errors() {
        // do not instrument specs if there are any errors in the spec population phase
        return Ok(env);
    }

    // TODO (mengxu): add the spec instrumentation logic.
    // Currently it just re-compiles the same AST again

    // Run the compiler fully on the instrumented program
    let units = match move_continue_up_to(
        None,
        MovePassResult::Expansion(expansion_ast, vec![]),
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
    let output_base = Path::new(output_dir);
    for unit in &verified_units {
        let (module, source_map, file_path) = match unit.clone() {
            CompiledUnit::Module {
                module, source_map, ..
            } => {
                let module_id = module.self_id();
                (
                    module,
                    source_map,
                    output_base
                        .join(MODULE_SUB_DIR)
                        .join(module_id.address().short_str_lossless())
                        .join(module_id.name().to_string()),
                )
            }
            CompiledUnit::Script {
                script,
                source_map,
                key,
                ..
            } => (
                script.into_module().1,
                source_map,
                output_base.join(SCRIPT_SUB_DIR).join(key),
            ),
        };
        // Dump the bytecode
        fs::create_dir_all(file_path.parent().unwrap())?;
        fs::write(
            file_path.with_extension(MOVE_COMPILED_EXTENSION),
            unit.serialize(),
        )?;
        // Disassemble the instrumented program if requested
        if dump_bytecode {
            let disas = Disassembler::new(
                SourceMapping::new(source_map, module),
                DisassemblerOptions {
                    only_externally_visible: false,
                    print_code: true,
                    print_basic_blocks: true,
                    print_locals: true,
                },
            );
            fs::write(
                file_path.with_extension(MOVE_DISASSEMBLY_EXTENSION),
                disas.disassemble()?,
            )?;
        }
    }

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
    target_sources: Vec<String>,
    other_sources: Vec<String>,
) -> anyhow::Result<(
    GlobalEnv,
    Option<(Vec<CompiledUnit>, Program, FilesSourceText)>,
)> {
    // Construct all sources from targets and others, as we need bytecode for all of them.
    let mut all_sources = target_sources;
    all_sources.extend(other_sources.clone());
    let mut env = GlobalEnv::new();
    // Parse the program
    let (files, pprog_and_comments_res) = move_parse(&all_sources, &[], None)?;
    for fname in files.keys().sorted() {
        let fsrc = &files[fname];
        env.add_source(fname, fsrc, other_sources.contains(&fname.to_string()));
    }
    // Add any documentation comments found by the Move compiler to the env.
    let (comment_map, parsed_prog) = match pprog_and_comments_res {
        Err(errors) => {
            add_move_lang_errors(&mut env, errors);
            return Ok((env, None));
        }
        Ok(res) => res,
    };
    for (fname, documentation) in comment_map {
        let file_id = env.get_file_id(fname).expect("file name defined");
        env.add_documentation(file_id, documentation);
    }
    // Run the compiler up to expansion and clone a copy of the expansion program ast
    let (expansion_ast, expansion_result) = match move_continue_up_to(
        None,
        MovePassResult::Parser(parsed_prog),
        MovePass::Expansion,
    ) {
        Err(errors) => {
            add_move_lang_errors(&mut env, errors);
            return Ok((env, None));
        }
        Ok(MovePassResult::Expansion(eprog, eerrors)) => {
            (eprog.clone(), MovePassResult::Expansion(eprog, eerrors))
        }
        Ok(_) => unreachable!(),
    };
    // Run the compiler fully to the compiled units
    let units = match move_continue_up_to(None, expansion_result, MovePass::Compilation) {
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
    Ok((env, Some((verified_units, expansion_ast, files))))
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
