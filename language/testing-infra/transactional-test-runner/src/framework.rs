// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use crate::tasks::{
    taskify, InitCommand, PublishCommand, RunCommand, SyntaxChoice, TaskCommand, TaskInput,
    ViewCommand,
};
use anyhow::*;
use move_binary_format::file_format::{CompiledModule, CompiledScript};
use move_command_line_common::{
    env::read_bool_env_var,
    files::{MOVE_EXTENSION, MOVE_IR_EXTENSION},
    testing::{format_diff, read_env_update_baseline, EXP_EXT},
};
use move_core_types::{
    account_address::AccountAddress,
    identifier::IdentStr,
    language_storage::{ModuleId, TypeTag},
    transaction_argument::TransactionArgument,
};
use move_lang::{
    compiled_unit::AnnotatedCompiledUnit,
    diagnostics::{Diagnostics, FilesSourceText},
    shared::AddressBytes,
    FullyCompiledProgram,
};
use move_symbol_pool::Symbol;
use std::{
    collections::{BTreeMap, VecDeque},
    fmt::Debug,
    io::Write,
    path::Path,
    result::Result::Ok,
};
use structopt::*;
use tempfile::NamedTempFile;

pub struct ProcessedModule {
    module: CompiledModule,
    interface_file: Option<(String, NamedTempFile)>,
}

pub struct CompiledState<'a> {
    pre_compiled_deps: Option<&'a FullyCompiledProgram>,
    compiled_module_named_address_mapping: BTreeMap<ModuleId, Symbol>,
    named_address_mapping: BTreeMap<Symbol, AddressBytes>,
    modules: BTreeMap<ModuleId, ProcessedModule>,
}

pub trait MoveTestAdapter<'a> {
    type ExtraPublishArgs;
    type ExtraRunArgs;
    type Subcommand;
    type ExtraInitArgs;

    fn compiled_state(&mut self) -> &mut CompiledState<'a>;
    fn default_syntax(&self) -> SyntaxChoice;
    fn init(
        default_syntax: SyntaxChoice,
        option: Option<&'a FullyCompiledProgram>,
        init_data: Option<TaskInput<(InitCommand, Self::ExtraInitArgs)>>,
    ) -> Self;
    fn publish_module(
        &mut self,
        module: CompiledModule,
        gas_budget: Option<u64>,
        extra: Self::ExtraPublishArgs,
    ) -> Result<()>;
    fn execute_script(
        &mut self,
        script: CompiledScript,
        type_args: Vec<TypeTag>,
        signers: Vec<AccountAddress>,
        args: Vec<TransactionArgument>,
        gas_budget: Option<u64>,
        extra: Self::ExtraRunArgs,
    ) -> Result<()>;
    fn call_function(
        &mut self,
        module: &ModuleId,
        function: &IdentStr,
        type_args: Vec<TypeTag>,
        signers: Vec<AccountAddress>,
        args: Vec<TransactionArgument>,
        gas_budget: Option<u64>,
        extra: Self::ExtraRunArgs,
    ) -> Result<()>;
    fn view_data(
        &mut self,
        address: AccountAddress,
        module: &ModuleId,
        resource: &IdentStr,
        type_args: Vec<TypeTag>,
    ) -> Result<String>;

    fn handle_subcommand(
        &mut self,
        subcommand: TaskInput<Self::Subcommand>,
    ) -> Result<Option<String>>;

    fn handle_command(
        &mut self,
        task: TaskInput<
            TaskCommand<
                Self::ExtraInitArgs,
                Self::ExtraPublishArgs,
                Self::ExtraRunArgs,
                Self::Subcommand,
            >,
        >,
    ) -> Result<Option<String>> {
        let TaskInput {
            command,
            name,
            number,
            start_line,
            command_lines_stop,
            stop_line,
            data,
        } = task;
        match command {
            TaskCommand::Init { .. } => {
                panic!("The 'init' command is optional. But if used, it must be the first command")
            }
            TaskCommand::Publish(PublishCommand { gas_budget, syntax }, extra_args) => {
                let syntax = syntax.unwrap_or_else(|| self.default_syntax());
                let data = match data {
                    Some(f) => f,
                    None => panic!(
                        "Expected a module text block following 'publish' starting on lines {}-{}",
                        start_line, command_lines_stop
                    ),
                };
                let data_path = data.path().to_str().unwrap();
                let state = self.compiled_state();
                let (named_addr_opt, module, warnings_opt) = match syntax {
                    SyntaxChoice::Source => {
                        let (unit, warnings_opt) = compile_source_unit(
                            state.pre_compiled_deps,
                            state.named_address_mapping.clone(),
                            &state.interface_files().cloned().collect::<Vec<_>>(),
                            data_path.to_owned(),
                        )?;
                        match unit {
                        AnnotatedCompiledUnit::Module(annot_module) =>  {
                            let (named_addr_opt, _id) = annot_module.module_id();
                            (named_addr_opt.map(|n| n.value), annot_module.named_module.module, warnings_opt)
                        }
                        AnnotatedCompiledUnit::Script(_) => panic!(
                            "Expected a module text block, not a script, following 'publish' starting on lines {}-{}",
                            start_line, command_lines_stop
                        ),
                    }
                    }
                    SyntaxChoice::IR => {
                        let module = compile_ir_module(state.dep_modules(), data_path)?;
                        (None, module, None)
                    }
                };
                state.add(named_addr_opt, module.clone());
                self.publish_module(module, gas_budget, extra_args)?;
                Ok(warnings_opt)
            }
            TaskCommand::Run(
                RunCommand {
                    signers,
                    args,
                    type_args,
                    gas_budget,
                    syntax,
                    name: None,
                },
                extra_args,
            ) => {
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
                let (script, warning_opt) = match syntax {
                    SyntaxChoice::Source => {
                        let (unit, warning_opt) = compile_source_unit(
                            state.pre_compiled_deps,
                            state.named_address_mapping.clone(),
                            &state.interface_files().cloned().collect::<Vec<_>>(),
                            data_path.to_owned(),
                        )?;
                        match unit {
                        AnnotatedCompiledUnit::Script(annot_script) => (annot_script.named_script.script, warning_opt),
                        AnnotatedCompiledUnit::Module(_) => panic!(
                            "Expected a script text block, not a module, following 'run' starting on lines {}-{}",
                            start_line, command_lines_stop
                        ),
                    }
                    }
                    SyntaxChoice::IR => (compile_ir_script(state.dep_modules(), data_path)?, None),
                };
                self.execute_script(script, type_args, signers, args, gas_budget, extra_args)?;
                Ok(warning_opt)
            }
            TaskCommand::Run(
                RunCommand {
                    signers,
                    args,
                    type_args,
                    gas_budget,
                    syntax,
                    name: Some((module, name)),
                },
                extra_args,
            ) => {
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
                    extra_args,
                )?;
                Ok(None)
            }
            TaskCommand::View(ViewCommand {
                address,
                resource: (module, name, type_arguments),
            }) => Ok(Some(self.view_data(
                address,
                &module,
                name.as_ident_str(),
                type_arguments,
            )?)),
            TaskCommand::Subcommand(c) => self.handle_subcommand(TaskInput {
                command: c,
                name,
                number,
                start_line,
                command_lines_stop,
                stop_line,
                data,
            }),
        }
    }
}

impl<'a> CompiledState<'a> {
    pub fn new(
        named_address_mapping: BTreeMap<impl Into<Symbol>, AddressBytes>,
        pre_compiled_deps: Option<&'a FullyCompiledProgram>,
    ) -> Self {
        let named_address_mapping = named_address_mapping
            .into_iter()
            .map(|(k, v)| (k.into(), v))
            .collect();
        let mut state = Self {
            pre_compiled_deps,
            modules: BTreeMap::new(),
            compiled_module_named_address_mapping: BTreeMap::new(),
            named_address_mapping,
        };
        if let Some(pcd) = pre_compiled_deps {
            for unit in &pcd.compiled {
                if let AnnotatedCompiledUnit::Module(annot_module) = unit {
                    let (named_addr_opt, _id) = annot_module.module_id();
                    state.add(
                        named_addr_opt.map(|n| n.value),
                        annot_module.named_module.module.clone(),
                    );
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
                &self.compiled_module_named_address_mapping,
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

    pub fn add(&mut self, named_addr_opt: Option<Symbol>, module: CompiledModule) {
        let id = module.self_id();
        if let Some(named_addr) = named_addr_opt {
            self.compiled_module_named_address_mapping
                .insert(id.clone(), named_addr);
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
    named_address_mapping: BTreeMap<Symbol, AddressBytes>,
    deps: &[String],
    path: String,
) -> Result<(AnnotatedCompiledUnit, Option<String>)> {
    fn rendered_diags(files: &FilesSourceText, diags: Diagnostics) -> Option<String> {
        if diags.is_empty() {
            return None;
        }

        let error_buffer = if read_bool_env_var(move_command_line_common::testing::PRETTY) {
            move_lang::diagnostics::report_diagnostics_to_color_buffer(files, diags)
        } else {
            move_lang::diagnostics::report_diagnostics_to_buffer(files, diags)
        };
        Some(String::from_utf8(error_buffer).unwrap())
    }

    use move_lang::PASS_COMPILATION;
    let (mut files, comments_and_compiler_res) = move_lang::Compiler::new(&[path], deps)
        .set_pre_compiled_lib_opt(pre_compiled_deps)
        .set_named_address_values(named_address_mapping)
        .run::<PASS_COMPILATION>()?;
    let units_or_diags = comments_and_compiler_res
        .map(|(_comments, move_compiler)| move_compiler.into_compiled_units());

    match units_or_diags {
        Err(diags) => {
            if let Some(pcd) = pre_compiled_deps {
                for (file_name, text) in &pcd.files {
                    // TODO This is bad. Rethink this when errors are redone
                    if !files.contains_key(file_name) {
                        files.insert(*file_name, text.clone());
                    }
                }
            }

            Err(anyhow!(rendered_diags(&files, diags).unwrap()))
        }
        Ok((mut units, warnings)) => {
            let warnings = rendered_diags(&files, warnings);
            let len = units.len();
            if len != 1 {
                panic!("Invalid input. Expected 1 compiled unit but got {}", len)
            }
            let unit = units.pop().unwrap();
            Ok((unit, warnings))
        }
    }
}

fn compile_ir_module<'a>(
    deps: impl Iterator<Item = &'a CompiledModule>,
    file_name: &str,
) -> Result<CompiledModule> {
    use compiler::Compiler as IRCompiler;
    let code = std::fs::read_to_string(file_name).unwrap();
    IRCompiler::new(deps.collect()).into_compiled_module(file_name, &code)
}

fn compile_ir_script<'a>(
    deps: impl Iterator<Item = &'a CompiledModule>,
    file_name: &str,
) -> Result<CompiledScript> {
    use compiler::Compiler as IRCompiler;
    let code = std::fs::read_to_string(file_name).unwrap();
    let (script, _) = IRCompiler::new(deps.collect())
        .into_compiled_script_and_source_map(file_name.into(), &code)?;
    Ok(script)
}

pub fn run_test_impl<'a, Adapter>(
    path: &Path,
    fully_compiled_program_opt: Option<&'a FullyCompiledProgram>,
) -> Result<(), Box<dyn std::error::Error>>
where
    Adapter: MoveTestAdapter<'a>,
    Adapter::ExtraInitArgs: StructOptInternal + Debug,
    Adapter::ExtraPublishArgs: StructOptInternal + Debug,
    Adapter::ExtraRunArgs: StructOptInternal + Debug,
    Adapter::Subcommand: StructOptInternal + Debug,
{
    let extension = path.extension().unwrap().to_str().unwrap();
    let default_syntax = if extension == MOVE_IR_EXTENSION {
        SyntaxChoice::IR
    } else {
        assert!(extension == MOVE_EXTENSION);
        SyntaxChoice::Source
    };
    let mut output = String::new();
    let mut tasks = taskify::<
        TaskCommand<
            Adapter::ExtraInitArgs,
            Adapter::ExtraPublishArgs,
            Adapter::ExtraRunArgs,
            Adapter::Subcommand,
        >,
    >(path)
    .unwrap()
    .into_iter()
    .collect::<VecDeque<_>>();
    assert!(!tasks.is_empty());
    let num_tasks = tasks.len();
    output.push_str(&format!(
        "processed {} task{}\n",
        num_tasks,
        if num_tasks > 1 { "s" } else { "" }
    ));

    let first_task = tasks.pop_front().unwrap();
    let init_opt = match &first_task.command {
        TaskCommand::Init(_, _) => Some(first_task.map(|known| match known {
            TaskCommand::Init(command, extra_args) => (command, extra_args),
            _ => unreachable!(),
        })),
        _ => {
            tasks.push_front(first_task);
            None
        }
    };
    let mut adapter = Adapter::init(default_syntax, fully_compiled_program_opt, init_opt);
    for task in tasks {
        handle_known_task(&mut output, &mut adapter, task);
    }
    handle_expected_output(path, output)?;
    Ok(())
}

fn handle_known_task<'a, Adapter: MoveTestAdapter<'a>>(
    output: &mut String,
    adapter: &mut Adapter,
    task: TaskInput<
        TaskCommand<
            Adapter::ExtraInitArgs,
            Adapter::ExtraPublishArgs,
            Adapter::ExtraRunArgs,
            Adapter::Subcommand,
        >,
    >,
) {
    let task_number = task.number;
    let task_name = task.name.to_owned();
    let start_line = task.start_line;
    let stop_line = task.stop_line;
    let result = adapter.handle_command(task);
    let result_string = match result {
        Ok(None) => return,
        Ok(Some(s)) => s,
        Err(e) => format!("Error: {}", e),
    };
    assert!(!result_string.is_empty());
    output.push_str(&format!(
        "\ntask {} '{}'. lines {}-{}:\n{}\n",
        task_number, task_name, start_line, stop_line, result_string
    ));
}

fn handle_expected_output(test_path: &Path, output: impl AsRef<str>) -> Result<()> {
    let output = output.as_ref();
    assert!(!output.is_empty());
    let exp_path = test_path.with_extension(EXP_EXT);

    if read_env_update_baseline() {
        std::fs::write(exp_path, output).unwrap();
        return Ok(());
    }

    if !exp_path.exists() {
        std::fs::write(&exp_path, "").unwrap();
    }
    let expected_output = std::fs::read_to_string(&exp_path).unwrap();
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
