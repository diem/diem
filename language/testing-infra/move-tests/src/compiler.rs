// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{bail, Result};
use move_core_types::account_address::AccountAddress;
use move_lang::{
    compiled_unit::CompiledUnit, errors::report_errors_to_buffer, interface_generator,
    move_compile_no_report, shared::Address,
};
use move_testing_base::ops::OpGetOutputStream;
use std::{convert::TryFrom, fmt::Write as FmtWrite, io::Write as IOWrite};
use tempfile::{tempdir, NamedTempFile};
use vm::file_format::{CompiledModule, CompiledScript};

use crate::{
    ops::{OpGetPrecompiledUnit, OpGetPublishedModules},
    types::{MoveSource, MoveUnit},
};

pub fn move_compile<I1, I2>(
    sender_opt: Option<AccountAddress>,
    targets: I1,
    deps: I2,
) -> Result<std::result::Result<Vec<CompiledUnit>, String>>
where
    I1: IntoIterator<Item = MoveSource>,
    I2: IntoIterator<Item = MoveSource>,
{
    // Create a temp directory and a container to hold the handles of the temp files.
    let tempdir = tempdir()?;
    let mut tempfiles = vec![];

    // Create a temp file for each piece of embedded code.
    let mut materialize_source_file = |src| -> Result<_> {
        Ok(match src {
            MoveSource::Path(path) => path,
            MoveSource::Embedded(text) => {
                let f = NamedTempFile::new_in(tempdir.path())?;
                f.reopen()?.write_all(text.as_bytes())?;
                let path = f.path().to_string_lossy().to_string();
                tempfiles.push(f);
                path
            }
        })
    };
    let targets = targets
        .into_iter()
        .map(&mut materialize_source_file)
        .collect::<Result<Vec<_>>>()?;
    let deps = deps
        .into_iter()
        .map(&mut materialize_source_file)
        .collect::<Result<Vec<_>>>()?;

    // Call `move_compile`. Note: `move_compile` will wrap the actual result inside an outer result
    // so this step should almost always succeed.
    let (files, compiled_units_or_errors) = move_compile_no_report(
        &targets,
        &deps,
        sender_opt.map(|addr| Address::try_from(&addr.to_u8()[..]).unwrap()),
        None,
    )?;

    // Check the compilation result:
    //   - If successful, add the bindings to the state.
    //   - If unsuccessful:
    //     - Abort if this compilation task is expected to succeed.
    //     - Otherwise render the move errors to the output and
    let compiler_res = match compiled_units_or_errors {
        Ok(units) => Ok(units),
        Err(errors) => {
            let buffer = report_errors_to_buffer(files, errors);
            Err(String::from_utf8(buffer).unwrap())
        }
    };

    Ok(compiler_res)
}

/// Parse a yaml-represented module or script.
pub fn move_compile_single_unit<S>(
    state: &mut S,
    sender_opt: Option<AccountAddress>,
    unit: MoveUnit,
) -> Result<Option<CompiledUnit>>
where
    S: OpGetOutputStream + OpGetPrecompiledUnit + OpGetPublishedModules,
{
    let compiled_unit_opt = match unit {
        MoveUnit::Precompiled(var) => Some(state.get_precompiled_unit(&var)?),
        MoveUnit::Source { source, deps } => {
            let deps = match deps {
                Some(deps) => deps,
                None => {
                    let modules = state.get_published_modules();
                    let mut deps = vec![];

                    for module_blob in modules {
                        let module = CompiledModule::deserialize(&module_blob).unwrap();
                        let (_, src) = interface_generator::generate(&module)?;
                        deps.push(MoveSource::Embedded(src))
                    }

                    deps
                }
            };
            let compiler_res = move_compile(sender_opt, vec![source], deps)?;

            match compiler_res {
                Ok(mut unit) => Some(unit.pop().unwrap()),
                Err(rendered_errors) => {
                    writeln!(state.get_output_stream(), "{}", rendered_errors)?;
                    None
                }
            }
        }
    };

    Ok(compiled_unit_opt)
}

pub fn expect_module(unit: CompiledUnit) -> Result<CompiledModule> {
    match unit {
        CompiledUnit::Module { module, .. } => Ok(module),
        CompiledUnit::Script { .. } => bail!("Expected module got script"),
    }
}

pub fn expect_script(unit: CompiledUnit) -> Result<CompiledScript> {
    match unit {
        CompiledUnit::Module { .. } => bail!("Expected script got module"),
        CompiledUnit::Script { script, .. } => Ok(script),
    }
}
