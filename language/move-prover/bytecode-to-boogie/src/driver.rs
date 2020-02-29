// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use crate::{
    boogie_wrapper::BoogieWrapper,
    bytecode_translator::BoogieTranslator,
    cli::{abort_on_error, Options, INLINE_PRELUDE},
    code_writer::CodeWriter,
    env::{GlobalEnv, ModuleIndex},
};
use bytecode_verifier::VerifiedModule;
use ir_to_bytecode::{compiler::compile_module, parser::parse_module};
use itertools::Itertools;
use libra_types::{account_address::AccountAddress, identifier::Identifier};
use log::info;
use move_ir_types::{
    ast::ModuleDefinition,
    location::Spanned,
    spec_language_ast::{Condition, Invariant},
};
use std::{collections::BTreeMap, fs, path::Path};
use vm::{
    access::ModuleAccess,
    file_format::{FunctionDefinitionIndex, StructDefinitionIndex},
};

/// Content of the default prelude.
const DEFAULT_PRELUDE: &[u8] = include_bytes!("prelude.bpl");

/// A module index which we use as marker that generated code belongs to the boogie prelude
/// in CodeWriter.
pub const PSEUDO_PRELUDE_MODULE: ModuleIndex = std::usize::MAX;

/// Represents the driver for translation. Owns the environment used for translation.
pub struct Driver {
    /// Environment used for the translation.
    env: GlobalEnv,

    /// Generated output.
    writer: CodeWriter,
}

impl Driver {
    pub fn new(options: Options) -> Self {
        Driver {
            env: GlobalEnv::new(options),
            writer: CodeWriter::new(PSEUDO_PRELUDE_MODULE, Spanned::unsafe_no_loc(()).loc),
        }
    }

    /// Runs standard translation. When this finishes, the generated code is in `self.writer`.
    pub fn run(&mut self) -> bool {
        let sources = self.env.options.mvir_sources.clone();
        self.load_modules(&sources);
        self.add_prelude();
        self.add_helpers();
        self.translate_modules();
        let mut has_errors = self.env.get_modules().any(|me| me.has_errors());
        if !has_errors {
            // write resulting code
            info!("writing boogie to {}", self.env.options.output_path);
            abort_on_error(
                self.writer
                    .process_result(|result| fs::write(&self.env.options.output_path, result)),
                "cannot write boogie file",
            );
            if !self.env.options.generate_only {
                has_errors = self
                    .new_boogie_wrapper()
                    .call_boogie_and_verify_output(&self.env.options.output_path);
            }
        }
        // Report any diagnostics.
        for module_env in self.env.get_modules() {
            has_errors = has_errors || module_env.has_errors();
            module_env.report_diagnostics();
        }
        has_errors
    }

    /// Runs translation in a test context. Instead of writing output to a file, returns
    /// a pair of the prelude and the actual translated code.
    pub fn run_for_test(&mut self) -> (String, String) {
        let sources = self.env.options.mvir_sources.clone();
        self.load_modules(&sources);
        self.add_prelude();
        let prelude_end = self.writer.process_result(|result| result.len());
        self.add_helpers();
        self.translate_modules();
        self.writer.process_result(|result| {
            (
                result[0..prelude_end].to_string(),
                result[prelude_end..].to_string(),
            )
        })
    }

    /// Compiles the list of mvir modules and adds them to the state. On errors, this
    /// reports and aborts. Nothing will be added to the output yet.
    pub fn load_modules(&mut self, file_names: &[String]) {
        // TODO: make the address configurable.
        let address = AccountAddress::default();
        for file_name in file_names {
            info!("analyzing {}", file_name);
            // Parse module.
            let code = abort_on_error(fs::read_to_string(file_name), "cannot read mvir file");
            let parsed_module =
                abort_on_error(parse_module(file_name, &code), "mvir parsing errors");

            // Extract information from parsed module.
            let mut func_infos = self.extract_function_infos(&parsed_module);
            let mut struct_infos = self.extract_struct_infos(&parsed_module);
            let synthetics = parsed_module.synthetics.clone();

            // Compile module.
            let (compiled_module, source_map) = abort_on_error(
                compile_module(address, parsed_module, self.env.get_bytecode_modules()),
                "mvir compilation errors",
            );

            // Verify module byte code.
            let verified_module = abort_on_error(
                VerifiedModule::new(compiled_module)
                    // As an error message, produce a line separated list of VMStatus.
                    .map_err(|(_, sv)| sv.iter().map(|s| format!("{:?}", s)).join("\n")),
                "mvir verification errors",
            );

            // Add module to environment.
            let struct_data = (0..verified_module.struct_defs().len())
                .map(|idx| {
                    let def_idx = StructDefinitionIndex(idx as u16);
                    self.env.create_struct_data(
                        &verified_module,
                        def_idx,
                        struct_infos.remove(&def_idx).unwrap(),
                    )
                })
                .collect();
            let function_data = (0..verified_module.function_defs().len())
                .map(|idx| {
                    let def_idx = FunctionDefinitionIndex(idx as u16);
                    let (arg_names, type_arg_names, specs) =
                        func_infos.remove(&def_idx).expect("function index");

                    self.env.create_function_data(
                        &verified_module,
                        def_idx,
                        arg_names
                            .into_iter()
                            .map(|s| Identifier::new(s).unwrap())
                            .collect(),
                        type_arg_names
                            .into_iter()
                            .map(|s| Identifier::new(s).unwrap())
                            .collect(),
                        specs,
                    )
                })
                .collect();
            self.env.add(
                file_name,
                verified_module,
                source_map,
                struct_data,
                function_data,
                synthetics,
            );
        }
    }

    /// Extract function infos from the given parsed module.
    fn extract_function_infos(
        &self,
        parsed_module: &ModuleDefinition,
    ) -> BTreeMap<FunctionDefinitionIndex, (Vec<String>, Vec<String>, Vec<Condition>)> {
        let mut result = BTreeMap::new();
        for (raw_index, (_, def)) in parsed_module.functions.iter().enumerate() {
            let type_arg_names = def
                .value
                .signature
                .type_formals
                .iter()
                .map(|(v, _)| v.value.name().into())
                .collect();
            let arg_names = def
                .value
                .signature
                .formals
                .iter()
                .map(|(v, _)| v.value.name().into())
                .collect();
            let index = FunctionDefinitionIndex(raw_index as u16);
            result.insert(
                index,
                (arg_names, type_arg_names, def.value.specifications.clone()),
            );
        }
        result
    }

    /// Extract struct infos from parsed module.
    fn extract_struct_infos(
        &self,
        parsed_module: &ModuleDefinition,
    ) -> BTreeMap<StructDefinitionIndex, Vec<Invariant>> {
        let mut result = BTreeMap::new();
        for (raw_index, def) in parsed_module.structs.iter().enumerate() {
            let index = StructDefinitionIndex(raw_index as u16);
            result.insert(index, def.value.invariants.clone());
        }
        result
    }

    /// Adds the prelude to the generated output.
    pub fn add_prelude(&mut self) {
        emit!(
            self.writer,
            "\n// ** prelude from {}\n\n",
            self.env.options.prelude_path
        );
        if self.env.options.prelude_path == INLINE_PRELUDE {
            info!("using inline prelude");
            emitln!(self.writer, &String::from_utf8_lossy(DEFAULT_PRELUDE));
        } else {
            info!("using prelude at {}", &self.env.options.prelude_path);
            let content = abort_on_error(
                fs::read_to_string(&self.env.options.prelude_path),
                "cannot read prelude file",
            );
            emitln!(self.writer, &content);
        }
    }

    /// Add boogie helper functions on per-source base. For every source `path.mvir`, if a file
    /// `path.prover.bpl` exists, add it to the boogie output.
    pub fn add_helpers(&mut self) {
        for src in &self.env.options.mvir_sources {
            let path = Path::new(src);
            let parent = path
                .parent()
                .unwrap_or_else(|| Path::new(""))
                .to_str()
                .unwrap()
                .to_string();
            let helper = &format!(
                "{}/{}.prover.bpl",
                if parent == "" {
                    ".".to_string()
                } else {
                    parent
                },
                path.file_stem()
                    .expect("file has no ending")
                    .to_string_lossy()
            );
            if let Ok(content) = fs::read_to_string(helper) {
                info!("reading helper functions from {}", helper);
                emitln!(self.writer, "\n// ** helpers from {}", helper);
                emit!(self.writer, &content);
            }
        }
    }

    /// Translates all modules.
    pub fn translate_modules(&mut self) {
        BoogieTranslator::new(&self.env, &self.writer).translate();
    }

    /// Creates a BoogieWrapper which allows to call boogie and analyze results.
    pub fn new_boogie_wrapper(&self) -> BoogieWrapper<'_> {
        BoogieWrapper {
            env: &self.env,
            writer: &self.writer,
        }
    }
}
