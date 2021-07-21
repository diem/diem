// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use crate::tasks::{taskify, ProvidedCommand, SyntaxChoice, TaskInput};
use anyhow::*;
use either::Either;
use move_binary_format::file_format::{CompiledModule, CompiledScript};
use move_command_line_common::{
    env::read_bool_env_var,
    testing::{format_diff, read_env_update_baseline, EXP_EXT},
};
use move_core_types::{
    account_address::AccountAddress,
    identifier::IdentStr,
    language_storage::{ModuleId, TypeTag},
    transaction_argument::TransactionArgument,
};
use move_lang::{compiled_unit::CompiledUnit, FullyCompiledProgram};
use std::{collections::BTreeMap, fmt::Debug, io::Write, path::Path};
use structopt::*;
use tempfile::NamedTempFile;

pub struct ProcessedModule {
    module: CompiledModule,
    interface_file: Option<(String, NamedTempFile)>,
}

pub struct CompiledState<'a> {
    pre_compiled_deps: Option<&'a FullyCompiledProgram>,
    named_addresses: BTreeMap<ModuleId, String>,
    modules: BTreeMap<ModuleId, ProcessedModule>,
}

pub trait MoveAdaptor<'a> {
    type Subcommand: Debug + StructOpt;

    fn compiled_state(&mut self) -> &mut CompiledState<'a>;
    fn default_syntax(&self) -> SyntaxChoice;

    fn init(option: Option<&'a FullyCompiledProgram>) -> Self;
    fn publish_module(&mut self, module: CompiledModule, gas_budget: Option<u64>) -> Result<()>;
    fn execute_script(
        &mut self,
        script: CompiledScript,
        type_args: Vec<TypeTag>,
        signers: Vec<AccountAddress>,
        args: Vec<TransactionArgument>,
        gas_budget: Option<u64>,
    ) -> Result<()>;
    fn call_function(
        &mut self,
        module: &ModuleId,
        function: &IdentStr,
        type_args: Vec<TypeTag>,
        signers: Vec<AccountAddress>,
        args: Vec<TransactionArgument>,
        gas_budget: Option<u64>,
    ) -> Result<()>;
    fn view_data(
        &mut self,
        address: AccountAddress,
        module: &ModuleId,
        resource: &IdentStr,
        type_args: Vec<TypeTag>,
    ) -> Result<String>;

    fn handle_subcommand(&mut self, subcommand: TaskInput<Self::Subcommand>) -> Result<String>;

    fn handle_default_command(&mut self, task: TaskInput<ProvidedCommand>) -> Result<String> {
        let TaskInput {
            command,
            number: _,
            start_line,
            command_lines_stop,
            stop_line: _,
            data,
        } = task;
        match command {
            ProvidedCommand::Publish {
                gas_budget,
                syntax,
                address,
            } => {
                let syntax = syntax.unwrap_or_else(|| self.default_syntax());
                match (syntax, address) {
                    (SyntaxChoice::Move, None) | (SyntaxChoice::IR, Some(_)) => (),
                    (SyntaxChoice::IR, None) => {
                        panic!("The address flag for 'publish' must be set for IR syntax")
                    }
                    (SyntaxChoice::Move, Some(_)) => panic!(
                        "The address flag for 'publish' cannot be set \
                        for Move source language syntax"
                    ),
                };
                let data = match data {
                    Some(f) => f,
                    None => panic!(
                        "Expected a module text block following 'publish' starting on lines {}-{}",
                        start_line, command_lines_stop
                    ),
                };
                let data_path = data.path().to_str().unwrap();
                let state = self.compiled_state();
                let (named_addr_opt, module) = match syntax {
                    SyntaxChoice::Move => {
                        let unit = compile_source_unit(
                            state.pre_compiled_deps,
                            &state.interface_files().cloned().collect::<Vec<_>>(),
                            data_path.to_owned(),
                        )?;
                        match unit {
                        CompiledUnit::Module { ident, module, .. } =>  {
                            let (named_addr_opt, _id) = ident.into_module_id();
                            (named_addr_opt.map(|n| n.value), module)
                        }
                        CompiledUnit::Script { .. } => panic!(
                            "Expected a module text block, not a script, following 'publish' starting on lines {}-{}",
                            start_line, command_lines_stop
                        ),
                    }
                    }
                    SyntaxChoice::IR => {
                        let module =
                            compile_ir_module(state.dep_modules(), address.unwrap(), data_path)?;
                        (None, module)
                    }
                };
                let id = module.self_id();
                state.add(named_addr_opt, module.clone());
                self.publish_module(module, gas_budget)?;
                Ok(format!(
                    "Successfully published module {}::{}",
                    id.address(),
                    id.name()
                ))
            }
            ProvidedCommand::Run {
                signers,
                args,
                type_args,
                gas_budget,
                syntax,
                name: None,
            } => {
                let syntax = syntax.unwrap_or_else(|| self.default_syntax());
                let data = match data {
                    Some(f) => f,
                    None => panic!(
                        "Expected a script text block following 'run' starting on lines {}-{}",
                        start_line, command_lines_stop
                    ),
                };
                let data_path = data.path().to_str().unwrap();
                let state = self.compiled_state();
                let script = match syntax {
                    SyntaxChoice::Move => {
                        let unit = compile_source_unit(
                            state.pre_compiled_deps,
                            &state.interface_files().cloned().collect::<Vec<_>>(),
                            data_path.to_owned(),
                        )?;
                        match unit {
                        CompiledUnit::Script { script, .. }=> script,
                        CompiledUnit::Module { .. } => panic!(
                            "Expected a script text block, not a module, following 'run' starting on lines {}-{}",
                            start_line, command_lines_stop
                        ),
                    }
                    }
                    SyntaxChoice::IR => compile_ir_script(state.dep_modules(), data_path)?,
                };
                self.execute_script(script, type_args, signers, args, gas_budget)?;
                Ok("Successfully executed script".to_owned())
            }
            ProvidedCommand::Run {
                signers,
                args,
                type_args,
                gas_budget,
                syntax,
                name: Some((module, name)),
            } => {
                assert!(
                    syntax.is_none(),
                    "syntax flag meaningless with function execution"
                );
                self.call_function(
                    &module,
                    name.as_ident_str(),
                    type_args,
                    signers,
                    args,
                    gas_budget,
                )?;
                Ok(format!(
                    "Successfully executed function {}::{}",
                    module, name,
                ))
            }
            ProvidedCommand::View {
                address,
                resource: (module, name, type_arguments),
            } => self.view_data(address, &module, name.as_ident_str(), type_arguments),
        }
    }
}

impl<'a> CompiledState<'a> {
    pub fn new(pre_compiled_deps: Option<&'a FullyCompiledProgram>) -> Self {
        let mut state = Self {
            pre_compiled_deps,
            modules: BTreeMap::new(),
            named_addresses: BTreeMap::new(),
        };
        if let Some(pcd) = pre_compiled_deps {
            for unit in &pcd.compiled {
                if let CompiledUnit::Module { ident, module, .. } = unit {
                    let (named_addr_opt, _id) = ident.clone().into_module_id();
                    state.add(named_addr_opt.map(|n| n.value), module.clone());
                }
            }
        }
        state
    }

    pub fn dep_modules(&self) -> impl Iterator<Item = &CompiledModule> {
        self.modules.values().map(|pmod| &pmod.module)
    }

    pub fn interface_files(&mut self) -> impl Iterator<Item = &String> {
        for pmod in self
            .modules
            .values_mut()
            .filter(|pmod| pmod.interface_file.is_none())
        {
            let file = NamedTempFile::new().unwrap();
            let path = file.path().to_str().unwrap().to_owned();
            let (_id, interface_text) = move_lang::interface_generator::write_module_to_string(
                &self.named_addresses,
                &pmod.module,
            )
            .unwrap();
            file.reopen()
                .unwrap()
                .write_all(interface_text.as_bytes())
                .unwrap();
            debug_assert!(pmod.interface_file.is_none());
            pmod.interface_file = Some((path, file))
        }
        self.modules
            .values()
            .map(|pmod| &pmod.interface_file.as_ref().unwrap().0)
    }

    pub fn add(&mut self, named_addr_opt: Option<String>, module: CompiledModule) {
        let id = module.self_id();
        if let Some(named_addr) = named_addr_opt {
            self.named_addresses.insert(id.clone(), named_addr);
        }

        let processed = ProcessedModule {
            module,
            interface_file: None,
        };
        self.modules.insert(id, processed);
    }
}

fn compile_source_unit(
    pre_compiled_deps: Option<&FullyCompiledProgram>,
    deps: &[String],
    path: String,
) -> Result<CompiledUnit> {
    use move_lang::PASS_COMPILATION;
    let (mut files, comments_and_compiler_res) = move_lang::Compiler::new(&[path], deps)
        .set_pre_compiled_lib_opt(pre_compiled_deps)
        .run::<PASS_COMPILATION>()?;
    let units_or_errors = comments_and_compiler_res
        .map(|(_comments, move_compiler)| move_compiler.into_compiled_units());

    match units_or_errors {
        Err(errors) => {
            if let Some(pcd) = pre_compiled_deps {
                for (file_name, text) in &pcd.files {
                    // TODO This is bad. Rethink this when errors are redone
                    if !files.contains_key(file_name) {
                        files.insert(&**file_name, text.clone());
                    }
                }
            }

            let error_buffer = if read_bool_env_var(move_command_line_common::testing::PRETTY) {
                move_lang::errors::report_errors_to_color_buffer(files, errors)
            } else {
                move_lang::errors::report_errors_to_buffer(files, errors)
            };
            Err(anyhow!(String::from_utf8(error_buffer).unwrap()))
        }
        Ok(mut units) => {
            let len = units.len();
            if len != 1 {
                panic!("Invalid input. Expected 1 compiled unit but got {}", len)
            }
            Ok(units.pop().unwrap())
        }
    }
}

fn compile_ir_module<'a>(
    deps: impl Iterator<Item = &'a CompiledModule>,
    address: AccountAddress,
    file_name: &str,
) -> Result<CompiledModule> {
    use compiler::Compiler as IRCompiler;
    let code = std::fs::read_to_string(file_name).unwrap();
    IRCompiler::new(address, deps.collect()).into_compiled_module(&file_name, &code)
}

fn compile_ir_script<'a>(
    deps: impl Iterator<Item = &'a CompiledModule>,
    file_name: &str,
) -> Result<CompiledScript> {
    use compiler::Compiler as IRCompiler;
    let code = std::fs::read_to_string(file_name).unwrap();
    let (script, _) = IRCompiler::new(/* unused */ AccountAddress::ZERO, deps.collect())
        .into_compiled_script_and_source_map(&file_name, &code)?;
    Ok(script)
}

pub fn run_test_impl<'a, Command, Adaptor, Split>(
    path: &Path,
    adaptor: &mut Adaptor,
    split_command: Split,
) -> Result<(), Box<dyn std::error::Error>>
where
    Command: Debug + StructOpt,
    Adaptor: MoveAdaptor<'a>,
    Split: Fn(Command) -> Either<ProvidedCommand, Adaptor::Subcommand>,
{
    // TaskOptions::from_args();
    let mut output = String::new();
    for task_input in taskify::<Command>(path).unwrap() {
        let task_number = task_input.number;
        let start_line = task_input.start_line;
        let stop_line = task_input.stop_line;
        let result = match task_input.split(&split_command) {
            Either::Left(provided) => adaptor.handle_default_command(provided),
            Either::Right(sub) => adaptor.handle_subcommand(sub),
        };
        let result_string = match result {
            Ok(s) => s,
            Err(e) => format!("Error: {}", e),
        };
        output.push_str(&format!(
            "task {} lines {}-{}:\n{}\n\n",
            task_number, start_line, stop_line, result_string
        ));
    }
    handle_expected_output(path, output)?;
    Ok(())
}

fn handle_expected_output(test_path: &Path, output: impl AsRef<str>) -> Result<()> {
    let output = output.as_ref();
    let exp_path = test_path.with_extension(EXP_EXT);
    if !exp_path.exists() {
        std::fs::write(&exp_path, "").unwrap();
    }

    if read_env_update_baseline() {
        std::fs::write(&exp_path, output).unwrap();
        return Ok(());
    }

    let expected_output = std::fs::read_to_string(exp_path).unwrap();
    if output != expected_output {
        let msg = format!(
            "Expected errors differ from actual errors:\n{}",
            format_diff(output, expected_output),
        );
        anyhow::bail!(msg)
    } else {
        Ok(())
    }
}
