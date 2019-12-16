// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use crate::cli::{abort_on_error, Options, INLINE_PRELUDE};
use crate::translator::{BoogieTranslator, FunctionInfo, ModuleInfo};
use bytecode_source_map::source_map::SourceMap;
use bytecode_verifier::VerifiedModule;
use ir_to_bytecode::compiler::compile_module;
use ir_to_bytecode::parser::ast::{Loc, ModuleDefinition};
use ir_to_bytecode::parser::parse_module;
use itertools::Itertools;
use libra_types::account_address::AccountAddress;
use log::{debug, error, info};
use std::fs;
use std::process::Command;

/// Content of the default prelude.
const DEFAULT_PRELUDE: &[u8] = include_bytes!("bytecode_instrs.bpl");

/// Represents the driver for translation.
pub struct Driver<'app> {
    /// Options passed via the cli.
    options: &'app Options,
    /// List of module specs.
    module_infos: Vec<ModuleInfo>,
    /// List of verified modules.
    verified_modules: Vec<VerifiedModule>,
    /// Source position map.
    source_map: SourceMap<Loc>,
    /// Generated output.
    pub output: String,
}

impl<'app> Driver<'app> {
    pub fn new(options: &'app Options) -> Self {
        Driver {
            options,
            module_infos: vec![],
            verified_modules: vec![],
            source_map: vec![],
            output: String::new(),
        }
    }

    /// Runs standard translation. When this finishes, the generated code is in `self.output`.
    pub fn run(&mut self) {
        self.load_modules(&self.options.mvir_sources);
        self.add_prelude();
        self.translate_modules();
        // write resulting code
        info!("writing boogie to {}", self.options.output_path);
        abort_on_error(
            fs::write(&self.options.output_path, &self.output),
            "cannot write boogie file",
        );
        if !self.options.generate_only {
            self.call_boogie(&self.options.output_path);
        }
    }

    /// Runs translation in a test context. Instead of writing output to a file, returns
    /// a pair of the prelude and the actual translated code.
    pub fn run_for_test(&mut self) -> (String, String) {
        self.load_modules(&self.options.mvir_sources);
        self.add_prelude();
        // Extract the prelude from the generated output, as we want to return it in a separate
        // string.
        let prelude = std::mem::replace(&mut self.output, String::new());
        self.translate_modules();
        (prelude, std::mem::replace(&mut self.output, String::new()))
    }

    /// Compiles the list of mvir modules and adds them to the state. On errors, this
    /// reports and aborts. Nothing will be added to the output yet.
    pub fn load_modules(&mut self, file_names: &[String]) {
        // TODO: make the address configurable.
        let address = AccountAddress::default();
        for (index, file_name) in file_names.iter().enumerate() {
            info!("analyzing {}", file_name);
            // Parse module.
            let code = abort_on_error(fs::read_to_string(file_name), "cannot read mvir file");
            let parsed_module = abort_on_error(parse_module(&code), "mvir parsing errors");
            let name = parsed_module.name.as_inner().to_string();

            // Extract specifications from module.
            let specs = self.extract_function_infos(&parsed_module);

            // Compile module.
            let (compiled_module, source_map) = abort_on_error(
                compile_module(address, parsed_module, &self.verified_modules),
                "mvir compilation errors",
            );

            // Verify module byte code.
            let verified_module = abort_on_error(
                VerifiedModule::new(compiled_module)
                    // As an error message, produce a line separated list of VMStatus.
                    .map_err(|(_, sv)| sv.iter().map(|s| format!("{:?}", s)).join("\n")),
                "mvir verification errors",
            );

            // Store result.
            self.verified_modules.push(verified_module);
            self.module_infos.push(ModuleInfo {
                index,
                name,
                function_infos: specs,
            });
            self.source_map.push(source_map);
        }
    }

    /// Extract function infos from the given parsed module.
    fn extract_function_infos(&self, parsed_module: &ModuleDefinition) -> Vec<FunctionInfo> {
        let mut result = vec![];
        for (index, (name, def)) in parsed_module.functions.iter().enumerate() {
            let function_name = name.as_inner().as_str().to_string();
            let type_arg_names = def
                .value
                .signature
                .type_formals
                .iter()
                .map(|(v, _)| v.value.name().as_str().to_string())
                .collect();
            let arg_names = def
                .value
                .signature
                .formals
                .iter()
                .map(|(v, _)| v.value.name().as_str().to_string())
                .collect();
            result.push(FunctionInfo {
                index,
                name: function_name,
                type_arg_names,
                arg_names,
                specification: def.value.specifications.clone(),
            });
        }
        result
    }

    /// Adds the prelude to the generated output.
    pub fn add_prelude(&mut self) {
        self.output.push_str(&format!(
            "\n// ** prelude from {}\n\n",
            self.options.prelude_path
        ));
        if self.options.prelude_path == INLINE_PRELUDE {
            info!("using inline prelude");
            self.output
                .push_str(&String::from_utf8_lossy(DEFAULT_PRELUDE));
        } else {
            info!("using prelude at {}", &self.options.prelude_path);
            let content = abort_on_error(
                fs::read_to_string(&self.options.prelude_path),
                "cannot read prelude file",
            );
            self.output.push_str(&content);
        }
    }

    /// Translates all modules.
    pub fn translate_modules(&mut self) {
        self.output.push_str(
            &BoogieTranslator::new(&self.verified_modules, &self.module_infos, &self.source_map)
                .translate(),
        );
    }

    /// Calls boogie on the given file.
    pub fn call_boogie(&self, boogie_file: &str) -> bool {
        let args = self.options.get_boogie_command(boogie_file);
        info!("calling boogie");
        debug!("command line: {}", args.iter().join(" "));
        let status = abort_on_error(
            Command::new(&args[0]).args(&args[1..]).status(),
            "error executing boogie",
        );
        if !status.success() {
            error!("boogie exited with status {}", status.code().unwrap());
            false
        } else {
            true
        }
    }
}
