// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use crate::bytecode_translator::BoogieTranslator;
use crate::cli::{abort_on_error, abort_with_error, Options, INLINE_PRELUDE};
use crate::env::GlobalEnv;
use bytecode_verifier::VerifiedModule;
use ir_to_bytecode::compiler::compile_module;
use ir_to_bytecode::parser::ast::ModuleDefinition;
use ir_to_bytecode::parser::parse_module;
use ir_to_bytecode_syntax::spec_language_ast::Condition;
use itertools::Itertools;
use libra_types::account_address::AccountAddress;
use libra_types::identifier::Identifier;
use log::{debug, error, info};
use regex::Regex;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::process::Command;
use vm::access::ModuleAccess;
use vm::file_format::{FunctionDefinitionIndex, StructDefinitionIndex};

/// Content of the default prelude.
const DEFAULT_PRELUDE: &[u8] = include_bytes!("prelude.bpl");

/// Represents the driver for translation. Owns the environment used for translation.
pub struct Driver {
    /// Environment used for the translation.
    env: GlobalEnv,

    /// Generated output.
    output: String,
}

impl Driver {
    pub fn new(options: Options) -> Self {
        Driver {
            env: GlobalEnv::new(options),
            output: String::new(),
        }
    }

    /// Runs standard translation. When this finishes, the generated code is in `self.output`.
    pub fn run(&mut self) {
        let sources = self.env.options.mvir_sources.clone();
        self.load_modules(&sources);
        self.add_prelude();
        self.add_helpers();
        self.translate_modules();
        // write resulting code
        info!("writing boogie to {}", self.env.options.output_path);
        abort_on_error(
            fs::write(&self.env.options.output_path, &self.output),
            "cannot write boogie file",
        );
        if !self.env.options.generate_only {
            self.call_boogie_and_verify_output(&self.env.options.output_path);
        }
    }

    /// Runs translation in a test context. Instead of writing output to a file, returns
    /// a pair of the prelude and the actual translated code.
    pub fn run_for_test(&mut self) -> (String, String) {
        let sources = self.env.options.mvir_sources.clone();
        self.load_modules(&sources);
        self.add_prelude();
        // Extract the prelude from the generated output, as we want to return it in a separate
        // string.
        let prelude = std::mem::replace(&mut self.output, String::new());
        self.add_helpers();
        self.translate_modules();
        (prelude, std::mem::replace(&mut self.output, String::new()))
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
            let parsed_module = abort_on_error(parse_module(&code), "mvir parsing errors");

            // Extract information from parsed module.
            let mut infos = self.extract_function_infos(&parsed_module);

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
                    self.env
                        .create_struct_data(&verified_module, StructDefinitionIndex(idx as u16))
                })
                .collect();
            let function_data = (0..verified_module.function_defs().len())
                .map(|idx| {
                    let def_idx = FunctionDefinitionIndex(idx as u16);
                    let (arg_names, type_arg_names, specs) =
                        infos.remove(&def_idx).expect("function index");

                    self.env.create_function_data(
                        &verified_module,
                        def_idx,
                        arg_names,
                        type_arg_names,
                        specs,
                    )
                })
                .collect();
            self.env
                .add(verified_module, source_map, struct_data, function_data);
        }
    }

    /// Extract function infos from the given parsed module.
    fn extract_function_infos(
        &self,
        parsed_module: &ModuleDefinition,
    ) -> BTreeMap<FunctionDefinitionIndex, (Vec<Identifier>, Vec<Identifier>, Vec<Condition>)> {
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

    /// Adds the prelude to the generated output.
    pub fn add_prelude(&mut self) {
        self.output.push_str(&format!(
            "\n// ** prelude from {}\n\n",
            self.env.options.prelude_path
        ));
        if self.env.options.prelude_path == INLINE_PRELUDE {
            info!("using inline prelude");
            self.output
                .push_str(&String::from_utf8_lossy(DEFAULT_PRELUDE));
        } else {
            info!("using prelude at {}", &self.env.options.prelude_path);
            let content = abort_on_error(
                fs::read_to_string(&self.env.options.prelude_path),
                "cannot read prelude file",
            );
            self.output.push_str(&content);
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
                self.output
                    .push_str(&format!("\n// ** helpers from {}", helper));
                self.output.push_str(&content);
            }
        }
    }

    /// Translates all modules.
    pub fn translate_modules(&mut self) {
        self.output
            .push_str(&BoogieTranslator::new(&self.env).translate());
    }

    /// Calls boogie on the given file. On success, returns a pair of a string with the standard
    /// output and a vector of lines in the output which contain boogie errors.
    pub fn call_boogie(&self, boogie_file: &str) -> Option<(String, Vec<String>)> {
        let args = self.env.options.get_boogie_command(boogie_file);
        info!("calling boogie");
        debug!("command line: {}", args.iter().join(" "));
        let output = abort_on_error(
            Command::new(&args[0]).args(&args[1..]).output(),
            "error executing boogie",
        );
        if !output.status.success() {
            error!("boogie exited with status {:?}", output.status.code());
            None
        } else {
            let out = String::from_utf8_lossy(&output.stdout).to_string();
            let mut diag = vec![];
            let diag_re = Regex::new(r"(?m)^.*(Error BP|Error:|error:).*$").unwrap();
            for cap in diag_re.captures_iter(&out) {
                diag.push(cap[0].to_string());
            }
            Some((out, diag))
        }
    }

    /// Calls boogie, analyzes output, and aborts if errors found.
    pub fn call_boogie_and_verify_output(&self, boogie_file: &str) {
        match self.call_boogie(boogie_file) {
            None => abort_with_error("exiting"), // we already reported boogie error
            Some((stdout, diag)) => {
                println!("{}", stdout);
                if !diag.is_empty() {
                    abort_with_error("boogie reported errors")
                }
            }
        }
    }
}
