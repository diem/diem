// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::errors::*;
use types::transaction::{Script, TransactionArgument};
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

/// Creates a transaction script by serializing the the given `CompiledProgram` and
/// bundling it with transaction arguments.
pub fn make_transaction_program(
    program: &CompiledProgram,
    args: &[TransactionArgument],
) -> Result<Script> {
    let (script_blob, _) = serialize_program(program)?;
    Ok(Script::new(script_blob, args.to_vec()))
}
