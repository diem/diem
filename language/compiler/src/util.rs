// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use ir_to_bytecode::{compiler::compile_module, parser::parse_module};
use std::{fs, path::Path};
use types::account_address::AccountAddress;
use vm::{access::ModuleAccess, file_format::CompiledModule};

pub fn do_compile_module<T: ModuleAccess>(
    source_path: &Path,
    address: AccountAddress,
    dependencies: &[T],
) -> CompiledModule {
    let source = fs::read_to_string(source_path)
        .unwrap_or_else(|_| panic!("Unable to read file: {:?}", source_path));
    let parsed_module = parse_module(&source).unwrap();
    compile_module(address, parsed_module, dependencies).unwrap()
}
