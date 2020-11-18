// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::path::Path;

use crate::{
    define_commands,
    framework::{run_test_impl, CompiledState, MoveAdaptor},
    tasks::{EmptySubcommand, SyntaxChoice, TaskInput},
};
use anyhow::*;
use either::Either;
use move_binary_format::{errors::VMResult, file_format::CompiledScript, CompiledModule};
use move_command_line_common::files::MOVE_IR_EXTENSION;
use move_core_types::{
    account_address::AccountAddress,
    identifier::IdentStr,
    language_storage::{ModuleId, StructTag, TypeTag},
    transaction_argument::{convert_txn_args, TransactionArgument},
};
use move_lang::{compiled_unit::CompiledUnit, FullyCompiledProgram};
use move_vm_runtime::{data_cache::MoveStorage, move_vm::MoveVM, session::Session};
use move_vm_test_utils::InMemoryStorage;
use move_vm_types::gas_schedule::GasStatus;
use once_cell::sync::Lazy;
use resource_viewer::MoveValueAnnotator;

define_commands!(TaskCommand);

const STD_ADDR: AccountAddress =
    AccountAddress::new([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1]);

struct SimpleVMAdaptor<'a> {
    compiled_state: CompiledState<'a>,
    storage: InMemoryStorage,
    default_syntax: Option<SyntaxChoice>,
}

impl<'a> MoveAdaptor<'a> for SimpleVMAdaptor<'a> {
    type Subcommand = EmptySubcommand;

    fn compiled_state(&mut self) -> &mut CompiledState<'a> {
        &mut self.compiled_state
    }

    fn default_syntax(&self) -> SyntaxChoice {
        self.default_syntax.unwrap()
    }

    fn init(pre_compiled_deps: Option<&'a FullyCompiledProgram>) -> Self {
        Self {
            compiled_state: CompiledState::new(pre_compiled_deps),
            default_syntax: None,
            storage: InMemoryStorage::new(),
        }
    }

    fn publish_module(&mut self, module: CompiledModule, gas_budget: Option<u64>) -> Result<()> {
        let mut module_bytes = vec![];
        module.serialize(&mut module_bytes)?;

        let id = module.self_id();
        let sender = *id.address();
        self.perform_session_action(gas_budget, |session, gas_status| {
            session.publish_module(module_bytes, sender, gas_status)
        })
        .map_err(|e| {
            anyhow!(
                "Unable to publish module '{}'. Got VMError: {:?}",
                module.self_id(),
                e
            )
        })
    }

    fn execute_script(
        &mut self,
        script: CompiledScript,
        type_args: Vec<TypeTag>,
        signers: Vec<AccountAddress>,
        txn_args: Vec<TransactionArgument>,
        gas_budget: Option<u64>,
    ) -> Result<()> {
        let mut script_bytes = vec![];
        script.serialize(&mut script_bytes)?;
        let args = convert_txn_args(&txn_args);
        self.perform_session_action(gas_budget, |session, gas_status| {
            session.execute_script(script_bytes, type_args, args, signers, gas_status)
        })
        .map_err(|e| anyhow!("Script execution failed with VMError: {:?}", e))
    }

    fn call_function(
        &mut self,
        module: &ModuleId,
        function: &IdentStr,
        type_args: Vec<TypeTag>,
        signers: Vec<AccountAddress>,
        txn_args: Vec<TransactionArgument>,
        gas_budget: Option<u64>,
    ) -> Result<()> {
        let args = convert_txn_args(&txn_args);
        self.perform_session_action(gas_budget, |session, gas_status| {
            session.execute_script_function(&module, function, type_args, args, signers, gas_status)
        })
        .map_err(|e| anyhow!("Function execution failed with VMError: {:?}", e))
    }

    fn view_data(
        &mut self,
        address: AccountAddress,
        module: &ModuleId,
        resource: &IdentStr,
        type_args: Vec<TypeTag>,
    ) -> Result<String> {
        let tag = StructTag {
            address: *module.address(),
            module: module.name().to_owned(),
            name: resource.to_owned(),
            type_params: type_args,
        };
        match self.storage.get_resource(&address, &tag).unwrap() {
            None => Ok("[No Resource Exists]".to_owned()),
            Some(data) => {
                let annotated =
                    MoveValueAnnotator::new(&self.storage).view_resource(&tag, &data)?;
                Ok(format!("{}", annotated))
            }
        }
    }

    fn handle_subcommand(&mut self, _subcommand: TaskInput<Self::Subcommand>) -> Result<String> {
        unreachable!()
    }
}

impl<'a> SimpleVMAdaptor<'a> {
    fn perform_session_action(
        &mut self,
        gas_budget: Option<u64>,
        f: impl FnOnce(&mut Session<InMemoryStorage>, &mut GasStatus) -> VMResult<()>,
    ) -> VMResult<()> {
        // start session
        let vm = MoveVM::new(move_stdlib::natives::all_natives(STD_ADDR)).unwrap();
        let (mut session, mut gas_status) = {
            let gas_status = move_cli::sandbox::utils::get_gas_status(gas_budget).unwrap();
            let session = vm.new_session(&self.storage);
            (session, gas_status)
        };

        // perform op
        f(&mut session, &mut gas_status)?;

        // save changeset
        // TODO support events
        let (changeset, _events) = session.finish()?;
        self.storage.apply(changeset).unwrap();
        Ok(())
    }
}

static PRECOMPILED_MOVE_STDLIB: Lazy<FullyCompiledProgram> = Lazy::new(|| {
    let program_res = move_lang::construct_pre_compiled_lib(
        &move_stdlib::move_stdlib_files(),
        None,
        move_lang::Flags::empty().set_sources_shadow_deps(false),
    )
    .unwrap();
    match program_res {
        Ok(stdlib) => stdlib,
        Err((files, errors)) => {
            eprintln!("!!!Standard library failed to compile!!!");
            move_lang::errors::report_errors(files, errors)
        }
    }
});

static MOVE_STDLIB_COMPILED: Lazy<Vec<CompiledModule>> = Lazy::new(|| {
    let (files, units_res) = move_lang::Compiler::new(&move_stdlib::move_stdlib_files(), &[])
        .build()
        .unwrap();
    match units_res {
        Ok(units) => units
            .into_iter()
            .filter_map(|m| match m {
                CompiledUnit::Module { module, .. } => Some(module),
                CompiledUnit::Script { .. } => None,
            })
            .collect(),
        Err(errors) => {
            eprintln!("!!!Standard library failed to compile!!!");
            move_lang::errors::report_errors(files, errors)
        }
    }
});

pub fn run_test(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    // TaskOptions::from_args();
    let mut adaptor = SimpleVMAdaptor::init(Some(&*PRECOMPILED_MOVE_STDLIB));
    let syntax = if path.extension().unwrap().to_str().unwrap() == MOVE_IR_EXTENSION {
        SyntaxChoice::IR
    } else {
        SyntaxChoice::Move
    };
    adaptor.default_syntax = Some(syntax);
    adaptor
        .perform_session_action(None, |session, gas_status| {
            for module in &*MOVE_STDLIB_COMPILED {
                let mut module_bytes = vec![];
                module.serialize(&mut module_bytes).unwrap();

                let id = module.self_id();
                let sender = *id.address();
                session
                    .publish_module(module_bytes, sender, gas_status)
                    .unwrap();
            }
            Ok(())
        })
        .unwrap();

    run_test_impl(path, &mut adaptor, |c| match c {
        TaskCommand::Normal(p) => Either::Left(p),
    })
}
