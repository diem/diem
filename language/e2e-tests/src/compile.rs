// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Support for compiling scripts and modules in tests.

use compiler::Compiler;

use bytecode_verifier::VerifiedModule;
use libra_types::{
    account_address::AccountAddress,
    transaction::{Module, Script, TransactionPayload},
};

/// Compile the provided Move code into a blob which can be used as the code to be published
/// (a Module).
pub fn compile_module_with_address(
    address: &AccountAddress,
    file_name: &str,
    code: &str,
) -> TransactionPayload {
    let compiler = Compiler {
        address: *address,
        ..Compiler::default()
    };
    TransactionPayload::Module(Module::new(
        compiler
            .into_module_blob(file_name, code)
            .expect("Module compilation failed"),
    ))
}

/// Compile the provided Move code into a blob which can be used as the code to be executed
/// (a Script).
pub fn compile_script_with_address(
    address: &AccountAddress,
    file_name: &str,
    code: &str,
    deps: Vec<VerifiedModule>,
) -> TransactionPayload {
    let extra_deps = deps
        .into_iter()
        .map(|verified_module| verified_module.into_inner())
        .collect();
    let compiler = Compiler {
        address: *address,
        extra_deps,
        ..Compiler::default()
    };
    TransactionPayload::Script(Script::new(
        compiler
            .into_script_blob(file_name, code)
            .expect("Script compilation failed"),
        vec![],
        vec![],
    ))
}
