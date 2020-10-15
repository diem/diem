// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::format_err;
use compiled_stdlib::{stdlib_modules, StdLibOptions};
use diem_state_view::StateView;
use diem_types::{
    account_address::AccountAddress,
    account_config::{self, diem_root_address},
    transaction::{ChangeSet, Script, TransactionArgument, Version},
};
use diem_vm::{data_cache::RemoteStorage, txn_effects_to_writeset_and_events};
use move_core_types::{
    gas_schedule::{CostTable, GasAlgebra, GasUnits},
    identifier::Identifier,
    language_storage::{ModuleId, TypeTag},
};
use move_vm_runtime::{
    data_cache::RemoteCache, logging::NoContextLog, move_vm::MoveVM, session::Session,
};
use move_vm_test_utils::{ChangeSet as MoveChanges, DeltaStorage};
use move_vm_types::{
    gas_schedule::{zero_cost_schedule, CostStrategy},
    values::Value,
};
use once_cell::sync::Lazy;
use vm::CompiledModule;

pub static ZERO_COST_SCHEDULE: Lazy<CostTable> = Lazy::new(zero_cost_schedule);

pub struct GenesisSession<'r, 'l, R>(Session<'r, 'l, R>);

impl<'r, 'l, R: RemoteCache> GenesisSession<'r, 'l, R> {
    pub fn exec_func(
        &mut self,
        sender: AccountAddress,
        module_name: &str,
        function_name: &str,
        ty_args: Vec<TypeTag>,
        args: Vec<Value>,
    ) {
        self.0
            .execute_function(
                &ModuleId::new(
                    account_config::CORE_CODE_ADDRESS,
                    Identifier::new(module_name).unwrap(),
                ),
                &Identifier::new(function_name).unwrap(),
                ty_args,
                args,
                sender,
                &mut CostStrategy::system(&ZERO_COST_SCHEDULE, GasUnits::new(100_000_000)),
                &NoContextLog::new(),
            )
            .unwrap_or_else(|e| {
                panic!(
                    "Error calling {}.{}: {}",
                    module_name,
                    function_name,
                    e.into_vm_status()
                )
            })
    }

    pub fn exec_script(&mut self, sender: AccountAddress, script: &Script) {
        self.0
            .execute_script(
                script.code().to_vec(),
                script.ty_args().to_vec(),
                convert_txn_args(script.args()),
                vec![sender],
                &mut CostStrategy::system(&ZERO_COST_SCHEDULE, GasUnits::new(100_000_000)),
                &NoContextLog::new(),
            )
            .unwrap()
    }

    fn disable_reconfiguration(&mut self) {
        self.exec_func(
            diem_root_address(),
            "DiemConfig",
            "disable_reconfiguration",
            vec![],
            vec![Value::transaction_argument_signer_reference(
                diem_root_address(),
            )],
        )
    }

    fn enable_reconfiguration(&mut self) {
        self.exec_func(
            diem_root_address(),
            "DiemConfig",
            "enable_reconfiguration",
            vec![],
            vec![Value::transaction_argument_signer_reference(
                diem_root_address(),
            )],
        )
    }
    pub fn set_diem_version(&mut self, version: Version) {
        self.exec_func(
            diem_root_address(),
            "DiemVersion",
            "set",
            vec![],
            vec![
                Value::transaction_argument_signer_reference(diem_root_address()),
                Value::u64(version),
            ],
        )
    }
}

/// Convert the transaction arguments into Move values.
fn convert_txn_args(args: &[TransactionArgument]) -> Vec<Value> {
    args.iter()
        .map(|arg| match arg {
            TransactionArgument::U8(i) => Value::u8(*i),
            TransactionArgument::U64(i) => Value::u64(*i),
            TransactionArgument::U128(i) => Value::u128(*i),
            TransactionArgument::Address(a) => Value::address(*a),
            TransactionArgument::Bool(b) => Value::bool(*b),
            TransactionArgument::U8Vector(v) => Value::vector_u8(v.clone()),
        })
        .collect()
}

fn move_module_changes(modules: &[CompiledModule]) -> MoveChanges {
    let mut shadow_changeset = MoveChanges::new();
    for module in modules.iter() {
        let id = module.self_id();
        let mut module_bytes = vec![];
        module.serialize(&mut module_bytes).unwrap();
        shadow_changeset.publish_or_overwrite_module(id, module_bytes);
    }
    shadow_changeset
}

pub fn build_changeset<S: StateView, F>(
    state_view: &S,
    procedure: F,
    modules: &[CompiledModule],
) -> ChangeSet
where
    F: FnOnce(&mut GenesisSession<DeltaStorage<RemoteStorage<S>>>),
{
    let move_vm = MoveVM::new();
    let move_changes = move_module_changes(modules);
    let mut effect = {
        let state_view_storage = RemoteStorage::new(state_view);
        let exec_storage = DeltaStorage::new(&state_view_storage, &move_changes);
        let mut session = GenesisSession(move_vm.new_session(&exec_storage));
        session.disable_reconfiguration();
        procedure(&mut session);
        session.enable_reconfiguration();
        session
            .0
            .finish()
            .map_err(|err| format_err!("Unexpected VM Error: {:?}", err))
            .unwrap()
    };

    for module in modules {
        let mut module_bytes = vec![];
        module.serialize(&mut module_bytes).unwrap();
        // TODO: Check compatibility between old and new modules.
        effect.modules.push((module.self_id(), module_bytes));
    }

    let (writeset, events) = txn_effects_to_writeset_and_events(effect)
        .map_err(|err| format_err!("Unexpected VM Error: {:?}", err))
        .unwrap();

    ChangeSet::new(writeset, events)
}

pub fn build_stdlib_upgrade_changeset<S: StateView>(state_view: &S) -> ChangeSet {
    build_changeset(state_view, |_| {}, stdlib_modules(StdLibOptions::Compiled))
}
