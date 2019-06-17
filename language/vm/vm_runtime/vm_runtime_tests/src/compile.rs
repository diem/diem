// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Support for compiling scripts and modules in tests.

use compiler::{
    compiler::{compile_module, compile_program as compiler_compile_program},
    parser::parse_program,
};
use stdlib::stdlib::*;
use types::{
    account_address::AccountAddress,
    transaction::{Program, TransactionArgument},
};
use vm::file_format::CompiledModule;

/// Compile the provided Move code into a blob which can be used as the code for a [`Program`].
///
/// The script is compiled with the default account address (`0x0`).
pub fn compile_script(code: &str) -> Vec<u8> {
    let address = AccountAddress::default();
    let parsed_program = parse_program(code).expect("program must parse");
    let deps = stdlib_deps(&address);
    let compiled_program =
        compiler_compile_program(&address, &parsed_program, &deps).expect("program must compile");

    let mut serialized_script = Vec::<u8>::new();
    compiled_program
        .script
        .serialize(&mut serialized_script)
        .expect("script must serialize");
    serialized_script
}

/// Compile the provided Move code and arguments into a `Program` using `address` as the
/// self address for any modules in `code`.
pub fn compile_program_with_address(
    address: &AccountAddress,
    code: &str,
    args: Vec<TransactionArgument>,
) -> Program {
    let deps = stdlib_deps(&AccountAddress::default());
    let parsed_program = parse_program(&code).expect("program must parse");
    let compiled_program =
        compiler_compile_program(address, &parsed_program, &deps).expect("program must compile");

    let mut serialized_script = Vec::<u8>::new();
    compiled_program
        .script
        .serialize(&mut serialized_script)
        .expect("script must serialize");
    let mut serialized_modules = vec![];
    for m in compiled_program.modules {
        let mut module = vec![];
        m.serialize(&mut module).expect("module must serialize");
        serialized_modules.push(module);
    }
    Program::new(serialized_script, serialized_modules, args)
}

/// Compile the provided Move code and arguments into a `Program`.
///
/// This supports both scripts and modules defined in the same Move code. The code is compiled with
/// the default account address (`0x0`).
pub fn compile_program(code: &str, args: Vec<TransactionArgument>) -> Program {
    let address = AccountAddress::default();
    compile_program_with_address(&address, code, args)
}

fn stdlib_deps(address: &AccountAddress) -> Vec<CompiledModule> {
    let coin_module = coin_module();
    let compiled_coin_module =
        compile_module(&address, &coin_module, &[]).expect("coin must compile");

    let hash_module = native_hash_module();
    let compiled_hash_module =
        compile_module(&address, &hash_module, &[]).expect("hash must compile");

    let account_module = account_module();

    let mut deps = vec![compiled_coin_module, compiled_hash_module];
    let compiled_account_module =
        compile_module(&address, &account_module, &deps).expect("account must compile");

    deps.push(compiled_account_module);
    deps
}
