// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Test infrastructure for the Libra VM.
//!
//! This crate contains helpers for executing tests against the Libra VM.

use bytecode_verifier::{VerifiedModule, VerifiedScript};
use compiler::Compiler;
use data_store::FakeDataStore;
use libra_types::{
    access_path::AccessPath,
    account_address::AccountAddress,
    transaction::{TransactionArgument, TransactionStatus},
    vm_error::VMStatus,
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
    println!("{}", address);
    let compiler = Compiler {
        address,
        ..Compiler::default()
    };
    let compiled_program = compiler
        .into_compiled_program(program)
        .expect("Failed to compile");
    let (verified_script, modules) =
        verify(&address, compiled_program.script, compiled_program.modules);
    execute(verified_script, args, modules)
}

pub fn execute(
    script: VerifiedScript,
    args: Vec<TransactionArgument>,
    modules: Vec<VerifiedModule>,
) -> VMResult<()> {
    // set up the DB
    let mut data_view = FakeDataStore::default();
    data_view.set(
        AccessPath::new(AccountAddress::random(), vec![]),
        vec![0, 0],
    );
    execute_function(script, modules, args, &data_view)
}

pub fn verify(
    sender_address: &AccountAddress,
    compiled_script: CompiledScript,
    modules: Vec<CompiledModule>,
) -> (VerifiedScript, Vec<VerifiedModule>) {
    let (verified_script, verified_modules) =
        static_verify_program(sender_address, compiled_script, modules)
            .expect("verification failure");
    (verified_script, verified_modules)
}

pub fn assert_status_eq(s1: &VMStatus, s2: &VMStatus) -> bool {
    assert_eq!(s1.major_status, s2.major_status);
    assert_eq!(s1.sub_status, s2.sub_status);
    true
}

pub fn transaction_status_eq(t1: &TransactionStatus, t2: &TransactionStatus) -> bool {
    match (t1, t2) {
        (TransactionStatus::Discard(s1), TransactionStatus::Discard(s2))
        | (TransactionStatus::Keep(s1), TransactionStatus::Keep(s2)) => assert_status_eq(s1, s2),
        _ => false,
    }
}

#[macro_export]
macro_rules! assert_prologue_parity {
    ($e1:expr, $e2:expr, $e3:expr) => {
        assert_status_eq(&$e1.unwrap(), &$e3);
        assert!(transaction_status_eq($e2, &TransactionStatus::Discard($e3)));
    };
}

#[macro_export]
macro_rules! assert_prologue_disparity {
    ($e1:expr => $e2:expr, $e3:expr => $e4:expr) => {
        assert_eq!($e1, $e2);
        assert!(transaction_status_eq($e3, &$e4));
    };
}
