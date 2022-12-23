// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{bail, Result};
use move_binary_format::file_format::{CompiledModule, CompiledScript};
use move_lang::{compiled_unit::CompiledUnit, Compiler as MoveCompiler, Flags};
use std::{fs::File, io::Write, path::Path};
use tempfile::tempdir;

pub fn compile_units(s: &str) -> Result<Vec<CompiledUnit>> {
    let dir = tempdir()?;

    let file_path = dir.path().join("modules.move");
    {
        let mut file = File::create(&file_path)?;
        writeln!(file, "{}", s)?;
    }

    let (_, units) = MoveCompiler::new(&[file_path.to_str().unwrap().to_string()], &[])
        .set_flags(Flags::empty().set_sources_shadow_deps(false))
        .set_named_address_values(move_stdlib::move_stdlib_named_addresses())
        .build_and_report()?;

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

pub fn compile_modules_in_file(path: &Path) -> Result<Vec<CompiledModule>> {
    let (_, units) = MoveCompiler::new(&[path.to_str().unwrap().to_string()], &[])
        .set_flags(Flags::empty().set_sources_shadow_deps(false))
        .build_and_report()?;

    expect_modules(units).collect()
}

#[allow(dead_code)]
pub fn compile_modules(s: &str) -> Result<Vec<CompiledModule>> {
    expect_modules(compile_units(s)?).collect()
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
