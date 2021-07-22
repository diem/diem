// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]
#![allow(dead_code)]

use codespan::ByteIndex;
use codespan_reporting::diagnostic::{Diagnostic, Label, LabelStyle};
use itertools::Itertools;
#[allow(unused_imports)]
use log::warn;
use std::collections::{BTreeMap, BTreeSet};

use builder::module_builder::ModuleBuilder;
use move_binary_format::{
    access::ModuleAccess,
    check_bounds::BoundsChecker,
    file_format::{
        self_module_name, AddressIdentifierIndex, CompiledModule, CompiledScript,
        FunctionDefinition, FunctionDefinitionIndex, FunctionHandle, FunctionHandleIndex,
        IdentifierIndex, ModuleHandle, ModuleHandleIndex, Signature, SignatureIndex,
        StructDefinitionIndex, Visibility,
    },
};
use move_core_types::{account_address::AccountAddress, identifier::Identifier};
use move_ir_types::location::sp;
use move_lang::{
    self,
    compiled_unit::{self, AnnotatedCompiledScript, AnnotatedCompiledUnit},
    diagnostics::Diagnostics,
    expansion::ast::{self as E, Address, ModuleDefinition, ModuleIdent, ModuleIdent_},
    parser::ast::{self as P, ModuleName as ParserModuleName},
    shared::{parse_named_address, unique_map::UniqueMap, AddressBytes},
    Compiler, Flags, PASS_COMPILATION, PASS_EXPANSION, PASS_PARSER,
};
use move_symbol_pool::Symbol as MoveStringSymbol;
use num::{BigUint, Num};

use crate::{
    ast::{ModuleName, Spec},
    builder::model_builder::ModelBuilder,
    model::{FunId, FunctionData, GlobalEnv, Loc, ModuleData, ModuleId, StructId},
    options::ModelBuilderOptions,
};

pub mod ast;
mod builder;
pub mod code_writer;
pub mod exp_generator;
pub mod exp_rewriter;
pub mod model;
pub mod native;
pub mod options;
pub mod pragmas;
pub mod spec_translator;
pub mod symbol;
pub mod ty;

// =================================================================================================
// Entry Point

/// Build the move model with default compilation flags and default options and no named addresses.
/// This collects transitive dependencies for move sources from the provided directory list.
pub fn run_model_builder(
    move_sources: &[String],
    deps_dir: &[String],
) -> anyhow::Result<GlobalEnv> {
    run_model_builder_with_options(
        move_sources,
        deps_dir,
        ModelBuilderOptions::default(),
        BTreeMap::new(),
    )
}

/// Build the move model with default compilation flags and custom options and a set of provided
/// named addreses.
/// This collects transitive dependencies for move sources from the provided directory list.
pub fn run_model_builder_with_options(
    move_sources: &[String],
    deps_dir: &[String],
    options: ModelBuilderOptions,
    named_address_mapping: BTreeMap<String, AddressBytes>,
) -> anyhow::Result<GlobalEnv> {
    run_model_builder_with_options_and_compilation_flags(
        move_sources,
        deps_dir,
        options,
        Flags::empty(),
        named_address_mapping,
    )
}

/// Build the move model with custom compilation flags and custom options
/// This collects transitive dependencies for move sources from the provided directory list.
pub fn run_model_builder_with_options_and_compilation_flags(
    move_sources: &[String],
    deps_dir: &[String],
    options: ModelBuilderOptions,
    flags: Flags,
    named_address_mapping: BTreeMap<String, AddressBytes>,
) -> anyhow::Result<GlobalEnv> {
    let mut env = GlobalEnv::new();
    env.set_extension(options);

    // Step 1: parse the program to get comments and a separation of targets and dependencies.
    let (files, comments_and_compiler_res) = Compiler::new(move_sources, deps_dir)
        .set_flags(flags)
        .set_named_address_values(named_address_mapping.clone())
        .run::<PASS_PARSER>()?;
    let (comment_map, compiler) = match comments_and_compiler_res {
        Err(diags) => {
            // Add source files so that the env knows how to translate locations of parse errors
            for fname in files.keys().sorted() {
                let fsrc = &files[fname];
                env.add_source(fname.as_str(), fsrc, /* is_dep */ false);
            }
            add_move_lang_diagnostics(&mut env, diags);
            return Ok(env);
        }
        Ok(res) => res,
    };
    let (compiler, parsed_prog) = compiler.into_ast();
    // Add source files for targets and dependencies
    let dep_files: BTreeSet<_> = parsed_prog
        .lib_definitions
        .iter()
        .map(|def| def.file())
        .collect();
    for fname in files.keys().sorted() {
        let fsrc = &files[fname];
        env.add_source(fname.as_str(), fsrc, dep_files.contains(fname));
    }

    // Add any documentation comments found by the Move compiler to the env.
    for (fname, documentation) in comment_map {
        let file_id = env.get_file_id(fname).expect("file name defined");
        env.add_documentation(
            file_id,
            documentation
                .into_iter()
                .map(|(idx, s)| (ByteIndex(idx), s))
                .collect(),
        )
    }

    // Step 2: run the compiler up to expansion
    let parsed_prog = {
        let P::Program {
            mut source_definitions,
            lib_definitions,
        } = parsed_prog;
        source_definitions.extend(lib_definitions);
        P::Program {
            source_definitions,
            lib_definitions: vec![],
        }
    };
    let (compiler, expansion_ast) = match compiler.at_parser(parsed_prog).run::<PASS_EXPANSION>() {
        Err(diags) => {
            add_move_lang_diagnostics(&mut env, diags);
            return Ok(env);
        }
        Ok(compiler) => compiler.into_ast(),
    };
    // Extract the module/script closure
    let mut visited_addresses = BTreeSet::new();
    let mut visited_modules = BTreeSet::new();
    for (_, mident, mdef) in &expansion_ast.modules {
        let src_file = mdef.loc.file();
        if !dep_files.contains(&src_file) {
            collect_related_modules_recursive(
                mident,
                &expansion_ast.modules,
                &mut visited_addresses,
                &mut visited_modules,
            );
        }
    }
    for sdef in expansion_ast.scripts.values() {
        let src_file = sdef.loc.file();
        if !dep_files.contains(&src_file) {
            for (_, mident, _neighbor) in &sdef.immediate_neighbors {
                collect_related_modules_recursive(
                    mident,
                    &expansion_ast.modules,
                    &mut visited_addresses,
                    &mut visited_modules,
                );
            }
            for addr in &sdef.used_addresses {
                if let Address::Named(n) = &addr {
                    visited_addresses.insert(&n.value);
                }
            }
        }
    }

    let addresses = named_address_mapping
        .into_iter()
        .filter_map(|(n, val)| {
            visited_addresses
                .contains(n.as_str())
                .then(|| (n.into(), val))
        })
        .collect();

    // Step 3: selective compilation.
    let expansion_ast = {
        let E::Program { modules, scripts } = expansion_ast;
        let modules = modules.filter_map(|mident, mut mdef| {
            visited_modules.contains(&mident.value).then(|| {
                mdef.is_source_module = true;
                mdef
            })
        });
        E::Program { modules, scripts }
    };
    // Run the compiler fully to the compiled units
    let units = match compiler
        .at_expansion(expansion_ast.clone())
        .run::<PASS_COMPILATION>()
    {
        Err(diags) => {
            add_move_lang_diagnostics(&mut env, diags);
            return Ok(env);
        }
        Ok(compiler) => {
            let (units, warnings) = compiler.into_compiled_units();
            if !warnings.is_empty() {
                // NOTE: these diagnostics are just warnings. it should be feasible to continue the
                // model building here. But before that, register the warnings to the `GlobalEnv`
                // first so we get a chance to report these warnings as well.
                add_move_lang_diagnostics(&mut env, warnings);
            }
            units
        }
    };
    // Check for bytecode verifier errors (there should not be any)
    let (verified_units, diags) = compiled_unit::verify_units(units);
    if !diags.is_empty() {
        add_move_lang_diagnostics(&mut env, diags);
        return Ok(env);
    }

    // Now that it is known that the program has no errors, run the spec checker on verified units
    // plus expanded AST. This will populate the environment including any errors.
    run_spec_checker(&mut env, addresses, verified_units, expansion_ast);
    Ok(env)
}

fn collect_used_addresses<'a>(
    used_addresses: &'a BTreeSet<Address>,
    visited_addresses: &mut BTreeSet<&'a str>,
) {
    for addr in used_addresses {
        if let Address::Named(n) = &addr {
            visited_addresses.insert(&n.value);
        }
    }
}

fn collect_related_modules_recursive<'a>(
    mident: &'a ModuleIdent_,
    modules: &'a UniqueMap<ModuleIdent, ModuleDefinition>,
    visited_addresses: &mut BTreeSet<&'a str>,
    visited_modules: &mut BTreeSet<ModuleIdent_>,
) {
    if visited_modules.contains(mident) {
        return;
    }
    let mdef = modules.get_(mident).unwrap();
    if let Address::Named(n) = &mident.address {
        visited_addresses.insert(&n.value);
    }
    collect_used_addresses(&mdef.used_addresses, visited_addresses);
    visited_modules.insert(*mident);
    for (_, next_mident, _) in &mdef.immediate_neighbors {
        collect_related_modules_recursive(next_mident, modules, visited_addresses, visited_modules);
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
            let data = env.create_struct_data(m, def_idx, symbol, Loc::default(), Spec::default());
            module_data.struct_data.insert(struct_id, data);
            module_data.struct_idx_to_id.insert(def_idx, struct_id);
        }

        env.module_data.push(module_data);
    }
    Ok(env)
}

fn add_move_lang_diagnostics(env: &mut GlobalEnv, diags: Diagnostics) {
    let mk_label = |is_primary: bool, (loc, msg): (move_ir_types::location::Loc, String)| {
        let style = if is_primary {
            LabelStyle::Primary
        } else {
            LabelStyle::Secondary
        };
        let loc = env.to_loc(&loc);
        Label::new(style, loc.file_id(), loc.span()).with_message(msg)
    };
    for (severity, msg, primary_label, secondary_labels) in diags.into_codespan_format() {
        let diag = Diagnostic::new(severity)
            .with_labels(vec![mk_label(true, primary_label)])
            .with_message(msg)
            .with_labels(
                secondary_labels
                    .into_iter()
                    .map(|e| mk_label(false, e))
                    .collect(),
            );
        env.add_diag(diag);
    }
}

#[allow(deprecated)]
fn script_into_module(compiled_script: CompiledScript) -> CompiledModule {
    let mut script = compiled_script;

    // Add the "<SELF>" identifier if it isn't present.
    //
    // Note: When adding an element to the table, in theory it is possible for the index
    // to overflow. This will not be a problem if we get rid of the script/module conversion.
    let self_ident_idx = match script
        .identifiers
        .iter()
        .position(|ident| ident.as_ident_str() == self_module_name())
    {
        Some(idx) => IdentifierIndex::new(idx as u16),
        None => {
            let idx = IdentifierIndex::new(script.identifiers.len() as u16);
            script
                .identifiers
                .push(Identifier::new(self_module_name().to_string()).unwrap());
            idx
        }
    };

    // Add a dummy adress if none exists.
    let dummy_addr = AccountAddress::new([0xff; AccountAddress::LENGTH]);
    let dummy_addr_idx = match script
        .address_identifiers
        .iter()
        .position(|addr| addr == &dummy_addr)
    {
        Some(idx) => AddressIdentifierIndex::new(idx as u16),
        None => {
            let idx = AddressIdentifierIndex::new(script.address_identifiers.len() as u16);
            script.address_identifiers.push(dummy_addr);
            idx
        }
    };

    // Add a self module handle.
    let self_module_handle_idx = match script
        .module_handles
        .iter()
        .position(|handle| handle.address == dummy_addr_idx && handle.name == self_ident_idx)
    {
        Some(idx) => ModuleHandleIndex::new(idx as u16),
        None => {
            let idx = ModuleHandleIndex::new(script.module_handles.len() as u16);
            script.module_handles.push(ModuleHandle {
                address: dummy_addr_idx,
                name: self_ident_idx,
            });
            idx
        }
    };

    // Find the index to the empty signature [].
    // Create one if it doesn't exist.
    let return_sig_idx = match script.signatures.iter().position(|sig| sig.0.is_empty()) {
        Some(idx) => SignatureIndex::new(idx as u16),
        None => {
            let idx = SignatureIndex::new(script.signatures.len() as u16);
            script.signatures.push(Signature(vec![]));
            idx
        }
    };

    // Create a function handle for the main function.
    let main_handle_idx = FunctionHandleIndex::new(script.function_handles.len() as u16);
    script.function_handles.push(FunctionHandle {
        module: self_module_handle_idx,
        name: self_ident_idx,
        parameters: script.parameters,
        return_: return_sig_idx,
        type_parameters: script.type_parameters,
    });

    // Create a function definition for the main function.
    let main_def = FunctionDefinition {
        function: main_handle_idx,
        visibility: Visibility::Script,
        acquires_global_resources: vec![],
        code: Some(script.code),
    };

    let module = CompiledModule {
        version: script.version,
        module_handles: script.module_handles,
        self_module_handle_idx,
        struct_handles: script.struct_handles,
        function_handles: script.function_handles,
        field_handles: vec![],
        friend_decls: vec![],

        struct_def_instantiations: vec![],
        function_instantiations: script.function_instantiations,
        field_instantiations: vec![],

        signatures: script.signatures,

        identifiers: script.identifiers,
        address_identifiers: script.address_identifiers,
        constant_pool: script.constant_pool,

        struct_defs: vec![],
        function_defs: vec![main_def],
    };
    BoundsChecker::verify_module(&module).expect("invalid bounds in module");
    module
}

#[allow(deprecated)]
fn run_spec_checker(
    env: &mut GlobalEnv,
    named_address_mapping: BTreeMap<MoveStringSymbol, AddressBytes>,
    units: Vec<AnnotatedCompiledUnit>,
    mut eprog: E::Program,
) {
    let mut builder = ModelBuilder::new(env, named_address_mapping);
    // Merge the compiled units with the expanded program, preserving the order of the compiled
    // units which is topological w.r.t. use relation.
    let modules = units
        .into_iter()
        .flat_map(|unit| {
            Some(match unit {
                AnnotatedCompiledUnit::Module(annot_module) => {
                    let module_ident = annot_module.module_ident();
                    let expanded_module = match eprog.modules.remove(&module_ident) {
                        Some(m) => m,
                        None => {
                            warn!(
                                "[internal] cannot associate bytecode module `{}` with AST",
                                module_ident
                            );
                            return None;
                        }
                    };
                    (
                        module_ident,
                        expanded_module,
                        annot_module.named_module.module,
                        annot_module.named_module.source_map,
                        annot_module.function_infos,
                    )
                }
                AnnotatedCompiledUnit::Script(AnnotatedCompiledScript {
                    loc: _loc,
                    named_script: script,
                    function_info,
                }) => {
                    let move_lang::expansion::ast::Script {
                        attributes,
                        loc,
                        immediate_neighbors,
                        used_addresses,
                        function_name,
                        constants,
                        function,
                        specs,
                    } = match eprog.scripts.remove(&script.name) {
                        Some(s) => s,
                        None => {
                            warn!(
                                "[internal] cannot associate bytecode script `{}` with AST",
                                script.name
                            );
                            return None;
                        }
                    };
                    // Convert the script into a module.
                    let address = Address::Anonymous(sp(loc, AddressBytes::DEFAULT_ERROR_BYTES));
                    let ident = sp(
                        loc,
                        ModuleIdent_::new(address, ParserModuleName(function_name.0)),
                    );
                    let mut function_infos = UniqueMap::new();
                    function_infos.add(function_name, function_info).unwrap();
                    // Construct a pseudo module definition.
                    let mut functions = UniqueMap::new();
                    functions.add(function_name, function).unwrap();
                    let expanded_module = ModuleDefinition {
                        attributes,
                        loc,
                        dependency_order: usize::MAX,
                        immediate_neighbors,
                        used_addresses,
                        is_source_module: true,
                        friends: UniqueMap::new(),
                        structs: UniqueMap::new(),
                        constants,
                        functions,
                        specs,
                    };
                    let module = script_into_module(script.script);
                    (
                        ident,
                        expanded_module,
                        module,
                        script.source_map,
                        function_infos,
                    )
                }
            })
        })
        .enumerate();
    for (module_count, (module_id, expanded_module, compiled_module, source_map, function_infos)) in
        modules
    {
        let loc = builder.to_loc(&expanded_module.loc);
        let addr_bytes = builder.resolve_address(&loc, &module_id.value.address);
        let module_name = ModuleName::from_address_bytes_and_name(
            addr_bytes,
            builder
                .env
                .symbol_pool()
                .make(&module_id.value.module.0.value),
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

pub fn parse_addresses_from_options(
    named_addr_strings: Vec<String>,
) -> anyhow::Result<BTreeMap<String, AddressBytes>> {
    named_addr_strings
        .iter()
        .map(|x| parse_named_address(x))
        .collect()
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
