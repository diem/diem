// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Support for compiling scripts and modules in tests.

use compiler::Compiler;
use libra_types::{
    account_address::AccountAddress,
    transaction::{Module, TransactionPayload},
};

/// Compile the provided Move code into a blob which can be used as the code for a [`Program`] or
/// a [`Script`].
///
/// The script is compiled with the default account address (`0x0`).
pub fn compile_script(code: &str) -> Vec<u8> {
    let compiler = Compiler {
        ..Compiler::default()
    };
    compiler.into_script_blob(code).unwrap()
}

/// Compile the provided Move code into a blob which can be used as a [`Script`].
pub fn compile_script_with_address(address: &AccountAddress, code: &str) -> Vec<u8> {
    let compiler = Compiler {
        address: *address,
        ..Compiler::default()
    };
    compiler.into_script_blob(code).unwrap()
}

/// Compile the provided Move code into a blob which can be used as the code to be published
/// (a Module).
pub fn compile_module_with_address(address: &AccountAddress, code: &str) -> TransactionPayload {
    let compiler = Compiler {
        address: *address,
        ..Compiler::default()
    };
    TransactionPayload::Module(Module::new(compiler.into_module_blob(code).unwrap()))
}
