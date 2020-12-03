// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{bail, Result};
use move_core_types::account_address::AccountAddress;
use move_lang::{compiled_unit::CompiledUnit, shared::Address};
use std::{fs::File, io::Write, path::Path};
use tempfile::tempdir;
use vm::file_format::{CompiledModule, CompiledScript};

pub fn compile_units(addr: AccountAddress, s: &str) -> Result<Vec<CompiledUnit>> {
    let dir = tempdir()?;

    let file_path = dir.path().join("modules.move");
    {
        let mut file = File::create(&file_path)?;
        writeln!(file, "{}", s)?;
    }

    let (_, units) = move_lang::move_compile_and_report(
        &[file_path.to_str().unwrap().to_string()],
        &[],
        Some(Address::new(addr.to_u8())),
        None,
    )?;

    dir.close()?;

    Ok(units)
}

fn expect_modules(
    units: impl IntoIterator<Item = CompiledUnit>,
) -> impl Iterator<Item = Result<CompiledModule>> {
    units.into_iter().map(|unit| match unit {
        CompiledUnit::Module { module, .. } => Ok(module),
        CompiledUnit::Script { .. } => bail!("expected modules got script"),
    })
}

pub fn compile_modules_in_file(addr: AccountAddress, path: &Path) -> Result<Vec<CompiledModule>> {
    let (_, units) = move_lang::move_compile_and_report(
        &[path.to_str().unwrap().to_string()],
        &[],
        Some(Address::new(addr.to_u8())),
        None,
    )?;

    expect_modules(units).collect()
}

#[allow(dead_code)]
pub fn compile_modules(addr: AccountAddress, s: &str) -> Result<Vec<CompiledModule>> {
    expect_modules(compile_units(addr, s)?).collect()
}

pub fn as_module(unit: CompiledUnit) -> CompiledModule {
    match unit {
        CompiledUnit::Module { module, .. } => module,
        CompiledUnit::Script { .. } => panic!("expected module got script"),
    }
}

pub fn as_script(unit: CompiledUnit) -> CompiledScript {
    match unit {
        CompiledUnit::Module { .. } => panic!("expected script got module"),
        CompiledUnit::Script { script, .. } => script,
    }
}
