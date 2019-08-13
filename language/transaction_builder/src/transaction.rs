// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::errors::*;
use types::transaction::{Program, TransactionArgument};
use vm::file_format::CompiledProgram;

/// Serializes the given script and modules to be published.
pub fn serialize_program(program: &CompiledProgram) -> Result<(Vec<u8>, Vec<Vec<u8>>)> {
    let mut script_blob = vec![];
    program.script.serialize(&mut script_blob)?;

    let module_blobs = program
        .modules
        .iter()
        .map(|m| {
            let mut module_blob = vec![];
            m.serialize(&mut module_blob)?;
            Ok(module_blob)
        })
        .collect::<Result<Vec<_>>>()?;

    Ok((script_blob, module_blobs))
}

/// Creates a transaction program by serializing the the given `CompiledProgram` and
/// bundling it with transaction arguments.
pub fn make_transaction_program(
    program: &CompiledProgram,
    args: &[TransactionArgument],
) -> Result<Program> {
    let (script_blob, module_blobs) = serialize_program(program)?;
    Ok(Program::new(script_blob, module_blobs, args.to_vec()))
}
