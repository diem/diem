// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Support for compiling scripts and modules in tests.

use compiler::Compiler;

use libra_types::{
    account_address::AccountAddress,
    transaction::{Module, TransactionPayload},
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
        compiler.into_module_blob(file_name, code).unwrap(),
    ))
}
