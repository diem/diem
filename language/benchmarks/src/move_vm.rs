// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use criterion::{measurement::Measurement, Criterion};
use move_binary_format::CompiledModule;
use move_core_types::{
    account_address::AccountAddress,
    identifier::{IdentStr, Identifier},
    language_storage::{ModuleId, CORE_CODE_ADDRESS},
};
use move_lang::{compiled_unit::CompiledUnit, Compiler, Flags};
use move_vm_runtime::move_vm::MoveVM;
use move_vm_test_utils::BlankStorage;
use move_vm_types::gas_schedule::GasStatus;
use once_cell::sync::Lazy;
use std::path::PathBuf;

static MOVE_BENCH_SRC_PATH: Lazy<PathBuf> = Lazy::new(|| {
    vec![env!("CARGO_MANIFEST_DIR"), "src", "bench.move"]
        .into_iter()
        .collect()
});

/// Entry point for the bench, provide a function name to invoke in Module Bench in bench.move.
pub fn bench<M: Measurement + 'static>(c: &mut Criterion<M>, fun: &str) {
    let modules = compile_modules();
    let move_vm = MoveVM::new(move_stdlib::natives::all_natives(
        AccountAddress::from_hex_literal("0x1").unwrap(),
    ))
    .unwrap();
    execute(c, &move_vm, modules, fun);
}

// Compile `bench.move` and its dependencies
fn compile_modules() -> Vec<CompiledModule> {
    let mut src_files = move_stdlib::move_stdlib_files();
    src_files.push(MOVE_BENCH_SRC_PATH.to_str().unwrap().to_owned());
    let (_files, compiled_units) = Compiler::new(&src_files, &[])
        .set_flags(Flags::empty().set_sources_shadow_deps(false))
        .set_named_address_values(move_stdlib::move_stdlib_named_addresses())
        .build_and_report()
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
    let storage = BlankStorage::new();
    let sender = CORE_CODE_ADDRESS;
    let mut session = move_vm.new_session(&storage);
    let mut gas_status = GasStatus::new_unmetered();

    for module in modules {
        let mut mod_blob = vec![];
        module
            .serialize(&mut mod_blob)
            .expect("Module serialization error");
        session
            .publish_module(mod_blob, sender, &mut gas_status)
            .expect("Module must load");
    }

    // module and function to call
    let module_id = ModuleId::new(sender, Identifier::new("Bench").unwrap());
    let fun_name = IdentStr::new(fun).unwrap_or_else(|_| panic!("Invalid identifier name {}", fun));

    // benchmark
    c.bench_function(fun, |b| {
        b.iter(|| {
            session
                .execute_function(&module_id, fun_name, vec![], vec![], &mut gas_status)
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
