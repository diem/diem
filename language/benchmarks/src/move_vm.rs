// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use bytecode_verifier::VerifiedModule;
use criterion::Criterion;
use libra_state_view::StateView;
use libra_types::{access_path::AccessPath, account_address::AccountAddress};
use libra_vm::data_cache::StateViewCache;
use move_core_types::{
    gas_schedule::{GasAlgebra, GasUnits},
    identifier::{IdentStr, Identifier},
    language_storage::ModuleId,
};
use move_lang::{compiled_unit::CompiledUnit, shared::Address};
use move_vm_runtime::{data_cache::TransactionDataCache, move_vm::MoveVM};
use move_vm_types::{
    gas_schedule::{zero_cost_schedule, CostStrategy},
    transaction_metadata::TransactionMetadata,
};
use std::path::PathBuf;

/// Entry point for the bench, provide a function name to invoke in Module Bench in bench.move.
pub fn bench(c: &mut Criterion, fun: &str) {
    let module = compile_module();
    let move_vm = MoveVM::new();
    execute(c, &move_vm, module, fun);
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
fn execute(c: &mut Criterion, move_vm: &MoveVM, module: VerifiedModule, fun: &str) {
    // establish running context
    let state = EmptyStateView;
    let gas_schedule = zero_cost_schedule();
    let data_cache = StateViewCache::new(&state);
    let mut data_store = TransactionDataCache::new(&data_cache);
    let mut cost_strategy = CostStrategy::transaction(&gas_schedule, GasUnits::new(100_000_000));
    let metadata = TransactionMetadata::default();

    move_vm
        .cache_module(module, &mut data_store)
        .expect("Module must load");

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
                    vec![],
                    vec![],
                    &mut cost_strategy,
                    &mut data_store,
                    &metadata,
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
