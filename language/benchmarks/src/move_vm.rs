// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use bytecode_verifier::VerifiedModule;
use criterion::Criterion;
use libra_state_view::StateView;
use libra_types::{
    access_path::AccessPath, account_address::AccountAddress, language_storage::ModuleId,
};
use move_core_types::{
    gas_schedule::{GasAlgebra, GasUnits},
    identifier::{IdentStr, Identifier},
};
use move_lang::{compiled_unit::CompiledUnit, shared::Address};
use move_vm_runtime::MoveVM;
use move_vm_state::{data_cache::BlockDataCache, execution_context::TransactionExecutionContext};
use std::path::PathBuf;
use vm::{gas_schedule::zero_cost_schedule, transaction_metadata::TransactionMetadata};

/// Entry point for the bench, provide a function name to invoke in Module Bench in bench.move.
pub fn bench(c: &mut Criterion, fun: &str) {
    let module = compile_module();
    let move_vm = MoveVM::new();
    move_vm.cache_module(module);
    execute(c, &move_vm, fun);
}

// Compile `bench.move`
fn compile_module() -> VerifiedModule {
    // TODO: this has only been tried with `cargo bench` from `libra/src/language/benchmarks`
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("src/bench.move");
    let s = path.to_str().expect("no path specified").to_owned();

    let (_, mut modules) =
        move_lang::move_compile(&[s], &[], Some(Address::default())).expect("Error compiling...");
    match modules.remove(0) {
        CompiledUnit::Module { module, .. } => {
            VerifiedModule::new(module).expect("Cannot verify code in file")
        }
        CompiledUnit::Script { .. } => panic!("Expected a module but received a script"),
    }
}

// execute a given function in the Bench module
fn execute(c: &mut Criterion, move_vm: &MoveVM, fun: &str) {
    // establish running context
    let state = EmptyStateView;
    let gas_schedule = zero_cost_schedule();
    let data_cache = BlockDataCache::new(&state);
    let mut interpreter_context =
        TransactionExecutionContext::new(GasUnits::new(100_000_000), &data_cache);
    let metadata = TransactionMetadata::default();

    // module and function to call
    let module_id = ModuleId::new(AccountAddress::default(), Identifier::new("Bench").unwrap());
    let fun_name = IdentStr::new(fun).unwrap_or_else(|_| panic!("Invalid identifier name {}", fun));

    // benchmark
    c.bench_function(fun, |b| {
        b.iter(|| {
            move_vm
                .execute_function(
                    &module_id,
                    &fun_name,
                    &gas_schedule,
                    &mut interpreter_context,
                    &metadata,
                    vec![],
                    vec![],
                )
                .unwrap_or_else(|_| panic!("Cannot execute function {}", fun))
        })
    });
}

//
// Utilities to get the VM going...
//

// An empty `StateView`
struct EmptyStateView;

impl StateView for EmptyStateView {
    fn get(&self, _access_path: &AccessPath) -> Result<Option<Vec<u8>>> {
        Ok(None)
    }

    fn multi_get(&self, _access_paths: &[AccessPath]) -> Result<Vec<Option<Vec<u8>>>> {
        unimplemented!()
    }

    fn is_genesis(&self) -> bool {
        true
    }
}
