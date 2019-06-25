// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Test infrastructure for the Libra VM.
//!
//! This crate contains helpers for executing tests against the Libra VM.

use compiler::Compiler;
use data_store::FakeDataStore;
use types::{
    access_path::AccessPath, account_address::AccountAddress, transaction::TransactionArgument,
};
use vm::{
    errors::*,
    file_format::{CompiledModule, CompiledScript},
};
use vm_runtime::{execute_function, static_verify_program};

#[cfg(test)]
mod tests;

pub mod account;
pub mod account_universe;
pub mod common_transactions;
pub mod compile;
pub mod data_store;
pub mod executor;
pub mod gas_costs;
mod proptest_types;

/// Compiles a program with the given arguments and executes it in the VM.
pub fn compile_and_execute(program: &str, args: Vec<TransactionArgument>) -> VMResult<()> {
    let address = AccountAddress::default();
    let compiler = Compiler {
        code: program,
        address,
        ..Compiler::default()
    };
    let compiled_program = compiler.into_compiled_program().expect("Failed to compile");
    let (compiled_script, modules) =
        verify(&address, compiled_program.script, compiled_program.modules);
    execute(compiled_script, args, modules)
}

pub fn execute(
    script: CompiledScript,
    args: Vec<TransactionArgument>,
    modules: Vec<CompiledModule>,
) -> VMResult<()> {
    // set up the DB
    let mut data_view = FakeDataStore::default();
    data_view.set(
        AccessPath::new(AccountAddress::random(), vec![]),
        vec![0, 0],
    );
    execute_function(script, modules, args, &data_view)
}

fn verify(
    sender_address: &AccountAddress,
    compiled_script: CompiledScript,
    modules: Vec<CompiledModule>,
) -> (CompiledScript, Vec<CompiledModule>) {
    let (verified_script, verified_modules, statuses) =
        static_verify_program(sender_address, compiled_script, modules);
    assert_eq!(statuses, vec![]);
    (verified_script, verified_modules)
}

#[macro_export]
macro_rules! assert_prologue_parity {
    ($e1:expr, $e2:expr, $e3:pat) => {
        assert_matches!($e1, Some($e3));
        assert_matches!($e2, TransactionStatus::Discard($e3));
    };
}

#[macro_export]
macro_rules! assert_prologue_disparity {
    ($e1:expr => $e2:pat, $e3:expr => $e4:pat) => {
        assert_matches!($e1, $e2);
        assert_matches!($e3, &$e4);
    };
}
