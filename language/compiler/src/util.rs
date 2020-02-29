// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Context;
use bytecode_source_map::source_map::ModuleSourceMap;
use ir_to_bytecode::{compiler::compile_module, parser::parse_module};
use libra_types::account_address::AccountAddress;
use move_ir_types::location::Loc;
use std::{fs, path::Path};
use vm::{access::ModuleAccess, file_format::CompiledModule};

pub fn do_compile_module<T: ModuleAccess>(
    source_path: &Path,
    address: AccountAddress,
    dependencies: &[T],
) -> (CompiledModule, ModuleSourceMap<Loc>) {
    let source = fs::read_to_string(source_path)
        .with_context(|| format!("Unable to read file: {:?}", source_path))
        .unwrap();
    let file = source_path.as_os_str().to_str().unwrap();
    let parsed_module = parse_module(file, &source).unwrap();
    compile_module(address, parsed_module, dependencies).unwrap()
}
