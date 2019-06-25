// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Support for compiling scripts and modules in tests.

use compiler::Compiler;
use types::{
    account_address::AccountAddress,
    transaction::{Program, TransactionArgument},
};

/// Compile the provided Move code into a blob which can be used as the code for a [`Program`].
///
/// The script is compiled with the default account address (`0x0`).
pub fn compile_script(code: &str) -> Vec<u8> {
    let compiler = Compiler {
        code,
        ..Compiler::default()
    };
    compiler.into_script_blob().unwrap()
}

/// Compile the provided Move code and arguments into a `Program` using `address` as the
/// self address for any modules in `code`.
pub fn compile_program_with_address(
    address: &AccountAddress,
    code: &str,
    args: Vec<TransactionArgument>,
) -> Program {
    let compiler = Compiler {
        address: *address,
        code,
        ..Compiler::default()
    };
    compiler.into_program(args).unwrap()
}

/// Compile the provided Move code and arguments into a `Program`.
///
/// This supports both scripts and modules defined in the same Move code. The code is compiled with
/// the default account address (`0x0`).
pub fn compile_program(code: &str, args: Vec<TransactionArgument>) -> Program {
    let compiler = Compiler {
        code,
        ..Compiler::default()
    };
    compiler.into_program(args).unwrap()
}
