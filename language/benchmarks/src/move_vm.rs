// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use criterion::{measurement::Measurement, Criterion};
use diem_state_view::StateView;
use diem_types::{access_path::AccessPath, account_address::AccountAddress};
use diem_vm::data_cache::StateViewCache;
use move_binary_format::CompiledModule;
use move_core_types::{
    identifier::{IdentStr, Identifier},
    language_storage::ModuleId,
};
use move_lang::{
    compiled_unit::CompiledUnit,
    shared::{Address, Flags},
};
use move_vm_runtime::{logging::NoContextLog, move_vm::MoveVM};
use move_vm_types::gas_schedule::GasStatus;
use once_cell::sync::Lazy;
use std::path::PathBuf;

static MOVE_BENCH_SRC_PATH: Lazy<PathBuf> = Lazy::new(|| {
    vec![env!("CARGO_MANIFEST_DIR"), "src", "bench.move"]
        .into_iter()
        .collect()
});

static STDLIB_VECTOR_SRC_PATH: Lazy<PathBuf> = Lazy::new(|| {
    vec![
        move_stdlib::move_stdlib_modules_full_path().as_str(),
        "Vector.move",
    ]
    .into_iter()
    .collect()
});

/// Entry point for the bench, provide a function name to invoke in Module Bench in bench.move.
pub fn bench<M: Measurement + 'static>(c: &mut Criterion<M>, fun: &str) {
    let modules = compile_modules();
    let move_vm = MoveVM::new();
    execute(c, &move_vm, modules, fun);
}

// Compile `bench.move` and its dependencies
fn compile_modules() -> Vec<CompiledModule> {
    let (_files, compiled_units) = move_lang::move_compile_and_report(
        &[
            STDLIB_VECTOR_SRC_PATH.to_str().unwrap().to_owned(),
            MOVE_BENCH_SRC_PATH.to_str().unwrap().to_owned(),
        ],
        &[],
        None,
        Flags::empty().set_sources_shadow_deps(false),
    )
    .expect("Error compiling...");
    compiled_units
        .into_iter()
        .map(|unit| match unit {
            CompiledUnit::Module { module, .. } => module,
            CompiledUnit::Script { .. } => panic!("Expected a module but received a script"),
        })
        .collect()
}

// execute a given function in the Bench module
fn execute<M: Measurement + 'static>(
    c: &mut Criterion<M>,
    move_vm: &MoveVM,
    modules: Vec<CompiledModule>,
    fun: &str,
) {
    // establish running context
    let sender = AccountAddress::new(Address::DIEM_CORE.to_u8());
    let state = EmptyStateView;
    let data_cache = StateViewCache::new(&state);
    let log_context = NoContextLog::new();
    let mut session = move_vm.new_session(&data_cache);
    let mut gas_status = GasStatus::new_unmetered();

    for module in modules {
        let mut mod_blob = vec![];
        module
            .serialize(&mut mod_blob)
            .expect("Module serialization error");
        session
            .publish_module(mod_blob, sender, &mut gas_status, &log_context)
            .expect("Module must load");
    }

    // module and function to call
    let module_id = ModuleId::new(sender, Identifier::new("Bench").unwrap());
    let fun_name = IdentStr::new(fun).unwrap_or_else(|_| panic!("Invalid identifier name {}", fun));

    // benchmark
    c.bench_function(fun, |b| {
        b.iter(|| {
            session
                .execute_function(
                    &module_id,
                    &fun_name,
                    vec![],
                    vec![],
                    &mut gas_status,
                    &log_context,
                )
                .unwrap_or_else(|err| {
                    panic!(
                        "{:?}::{} failed with {:?}",
                        &module_id,
                        fun,
                        err.into_vm_status()
                    )
                })
        })
    });
}

//
// Utilities to get the VM going...
//

// An empty `StateView`
struct EmptyStateView;

impl StateView for EmptyStateView {
    fn get(&self, _: &AccessPath) -> Result<Option<Vec<u8>>> {
        Ok(None)
    }

    fn is_genesis(&self) -> bool {
        true
    }
}
