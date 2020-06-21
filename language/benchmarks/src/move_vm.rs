// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
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
use move_vm_types::gas_schedule::{zero_cost_schedule, CostStrategy};
use std::path::PathBuf;
use vm::CompiledModule;

/// Entry point for the bench, provide a function name to invoke in Module Bench in bench.move.
pub fn bench(c: &mut Criterion, fun: &str) {
    let move_vm = MoveVM::new();

    let addr = [0u8; AccountAddress::LENGTH];
    let module = compile_module(&addr);
    let sender = AccountAddress::new(addr);
    execute(c, &move_vm, sender, module, fun);
}

// Compile `bench.move`
fn compile_module(addr: &[u8; AccountAddress::LENGTH]) -> CompiledModule {
    // TODO: this has only been tried with `cargo bench` from `libra/src/language/benchmarks`
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("src/bench.move");
    let s = path.to_str().expect("no path specified").to_owned();

    let (_, mut modules) =
        move_lang::move_compile(&[s], &[], Some(Address::new(*addr))).expect("Error compiling...");
    match modules.remove(0) {
        CompiledUnit::Module { module, .. } => module,
        CompiledUnit::Script { .. } => panic!("Expected a module but received a script"),
    }
}

// execute a given function in the Bench module
fn execute(
    c: &mut Criterion,
    move_vm: &MoveVM,
    sender: AccountAddress,
    module: CompiledModule,
    fun: &str,
) {
    // establish running context
    let state = EmptyStateView;
    let gas_schedule = zero_cost_schedule();
    let data_cache = StateViewCache::new(&state);
    let mut data_store = TransactionDataCache::new(&data_cache);
    let mut cost_strategy = CostStrategy::system(&gas_schedule, GasUnits::new(100_000_000));

    let mut mod_blob = vec![];
    module
        .serialize(&mut mod_blob)
        .expect("Module serialization error");
    move_vm
        .publish_module(mod_blob, sender, &mut data_store, &mut cost_strategy)
        .expect("Module must load");

    // module and function to call
    let module_id = ModuleId::new(AccountAddress::ZERO, Identifier::new("Bench").unwrap());
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
                    sender,
                    &mut data_store,
                    &mut cost_strategy,
                )
                .unwrap_or_else(|err| panic!("{:?}::{} failed with {:?}", &module_id, fun, err))
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

    fn multi_get(&self, _access_paths: &[AccessPath]) -> Result<Vec<Option<Vec<u8>>>> {
        unimplemented!()
    }

    fn is_genesis(&self) -> bool {
        true
    }
}
