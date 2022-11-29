// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Context;
use bytecode_source_map::source_map::SourceMap;
use ir_to_bytecode::{
    compiler::{compile_module, compile_script},
    parser::{parse_module, parse_script},
};
use move_binary_format::file_format::{CompiledModule, CompiledScript};
use move_core_types::account_address::AccountAddress;
use move_ir_types::location::Loc;
use move_symbol_pool::Symbol;
use std::{fs, path::Path};

pub fn do_compile_script(
    source_path: &Path,
    address: AccountAddress,
    dependencies: &[CompiledModule],
) -> (CompiledScript, SourceMap<Loc>) {
    let source = fs::read_to_string(source_path)
        .with_context(|| format!("Unable to read file: {:?}", source_path))
        .unwrap();
    let file = Symbol::from(source_path.as_os_str().to_str().unwrap());
    let parsed_script = parse_script(file, &source).unwrap();
    compile_script(Some(address), parsed_script, dependencies).unwrap()
}

pub fn do_compile_module(
    source_path: &Path,
    address: AccountAddress,
    dependencies: &[CompiledModule],
) -> (CompiledModule, SourceMap<Loc>) {
    let source = fs::read_to_string(source_path)
        .with_context(|| format!("Unable to read file: {:?}", source_path))
        .unwrap();
    let file = Symbol::from(source_path.as_os_str().to_str().unwrap());
    let parsed_module = parse_module(file, &source).unwrap();
    compile_module(address, parsed_module, dependencies).unwrap()
}
