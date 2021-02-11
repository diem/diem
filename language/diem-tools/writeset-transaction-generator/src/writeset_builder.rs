// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::format_err;
use compiled_stdlib::{stdlib_modules, StdLibModules, StdLibOptions};
use diem_state_view::StateView;
use diem_types::{
    account_address::AccountAddress,
    account_config::{self, diem_root_address},
    transaction::{ChangeSet, Script, Version},
};
use diem_vm::{convert_changeset_and_events, data_cache::RemoteStorage};
use move_core_types::{
    effects::ChangeSet as MoveChanges,
    gas_schedule::{CostTable, GasAlgebra, GasUnits},
    identifier::Identifier,
    language_storage::{ModuleId, TypeTag},
    transaction_argument::convert_txn_args,
    value::{serialize_values, MoveValue},
};
use move_vm_runtime::{
    data_cache::RemoteCache, logging::NoContextLog, move_vm::MoveVM, session::Session,
};
use move_vm_test_utils::DeltaStorage;
use move_vm_types::gas_schedule::{zero_cost_schedule, CostStrategy};
use once_cell::sync::Lazy;
use vm::CompiledModule;

pub static ZERO_COST_SCHEDULE: Lazy<CostTable> = Lazy::new(zero_cost_schedule);

pub struct GenesisSession<'r, 'l, R>(Session<'r, 'l, R>);

impl<'r, 'l, R: RemoteCache> GenesisSession<'r, 'l, R> {
    pub fn exec_func(
        &mut self,
        module_name: &str,
        function_name: &str,
        ty_args: Vec<TypeTag>,
        args: Vec<Vec<u8>>,
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
            "DiemConfig",
            "disable_reconfiguration",
            vec![],
            serialize_values(&vec![MoveValue::Signer(diem_root_address())]),
        )
    }

    fn enable_reconfiguration(&mut self) {
        self.exec_func(
            "DiemConfig",
            "enable_reconfiguration",
            vec![],
            serialize_values(&vec![MoveValue::Signer(diem_root_address())]),
        )
    }
    pub fn set_diem_version(&mut self, version: Version) {
        self.exec_func(
            "DiemVersion",
            "set",
            vec![],
            serialize_values(&vec![
                MoveValue::Signer(diem_root_address()),
                MoveValue::U64(version),
            ]),
        )
    }
}

fn move_module_changes<'a>(modules: impl IntoIterator<Item = &'a CompiledModule>) -> MoveChanges {
    let mut shadow_changeset = MoveChanges::new();
    for module in modules {
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
    bytes: &[Vec<u8>],
    modules: &[CompiledModule],
) -> ChangeSet
where
    F: FnOnce(&mut GenesisSession<DeltaStorage<RemoteStorage<S>>>),
{
    let move_vm = MoveVM::new();
    let move_changes = move_module_changes(modules);
    let (mut changeset, events) = {
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

    for (module, bytes) in modules.iter().zip(bytes) {
        // TODO: Check compatibility between old and new modules.
        changeset.publish_or_overwrite_module(module.self_id(), bytes.clone())
    }

    let (writeset, events) = convert_changeset_and_events(changeset, events)
        .map_err(|err| format_err!("Unexpected VM Error: {:?}", err))
        .unwrap();

    ChangeSet::new(writeset, events)
}

pub fn build_stdlib_upgrade_changeset<S: StateView>(state_view: &S) -> ChangeSet {
    let StdLibModules {
        bytes_opt,
        compiled_modules,
    } = stdlib_modules(StdLibOptions::Compiled);
    build_changeset(state_view, |_| {}, bytes_opt.unwrap(), compiled_modules)
}
