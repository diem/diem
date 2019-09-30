// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_ir_to_bytecode::{compiler::compile_module, parser::parse_module};
use libra_types::account_address::AccountAddress;
use libra_vm::{access::ModuleAccess, file_format::CompiledModule};
use std::{fs, path::Path};

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
