// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::util::build_stdlib;
use bytecode_verifier::verifier::{verify_module, verify_script};
use failure::prelude::*;
use ir_to_bytecode::{
    compiler::{compile_module, compile_program},
    parser::{parse_module, parse_program},
};
use types::account_address::AccountAddress;
use vm::{
    access::ScriptAccess,
    errors::VerificationError,
    file_format::{CompiledModule, CompiledScript},
};

#[allow(unused_macros)]
macro_rules! instr_count {
    ($compiled: expr, $instr: pat) => {
        $compiled
            .as_inner()
            .main
            .code
            .code
            .iter()
            .filter(|ins| match ins {
                $instr => true,
                _ => false,
            })
            .count();
    };
}

fn compile_script_string_impl(
    code: &str,
    deps: Vec<CompiledModule>,
) -> Result<(CompiledScript, Vec<VerificationError>)> {
    let parsed_program = parse_program(code).unwrap();
    let compiled_program = compile_program(&AccountAddress::default(), &parsed_program, &deps)?;

    let mut serialized_script = Vec::<u8>::new();
    compiled_program.script.serialize(&mut serialized_script)?;
    let deserialized_script = CompiledScript::deserialize(&serialized_script)?;
    assert_eq!(compiled_program.script, deserialized_script);

    Ok(verify_script(compiled_program.script))
}

pub fn compile_script_string_and_assert_no_error(
    code: &str,
    deps: Vec<CompiledModule>,
) -> Result<CompiledScript> {
    let (verified_script, verification_errors) = compile_script_string_impl(code, deps)?;
    assert!(verification_errors.is_empty());
    Ok(verified_script)
}

pub fn compile_script_string(code: &str) -> Result<CompiledScript> {
    compile_script_string_and_assert_no_error(code, vec![])
}

#[allow(dead_code)]
pub fn compile_script_string_with_deps(
    code: &str,
    deps: Vec<CompiledModule>,
) -> Result<CompiledScript> {
    compile_script_string_and_assert_no_error(code, deps)
}

#[allow(dead_code)]
pub fn compile_script_string_and_assert_error(
    code: &str,
    deps: Vec<CompiledModule>,
) -> Result<CompiledScript> {
    let (verified_script, verification_errors) = compile_script_string_impl(code, deps)?;
    assert!(!verification_errors.is_empty());
    Ok(verified_script)
}

fn compile_module_string_impl(
    code: &str,
    deps: Vec<CompiledModule>,
) -> Result<(CompiledModule, Vec<VerificationError>)> {
    let address = &AccountAddress::default();
    let module = parse_module(code).unwrap();
    let compiled_module = compile_module(&address, &module, &deps)?;

    let mut serialized_module = Vec::<u8>::new();
    compiled_module.serialize(&mut serialized_module)?;
    let deserialized_module = CompiledModule::deserialize(&serialized_module)?;
    assert_eq!(compiled_module, deserialized_module);

    Ok(verify_module(compiled_module))
}

pub fn compile_module_string_and_assert_no_error(
    code: &str,
    deps: Vec<CompiledModule>,
) -> Result<CompiledModule> {
    let (verified_module, verification_errors) = compile_module_string_impl(code, deps)?;
    assert!(verification_errors.is_empty());
    Ok(verified_module)
}

pub fn compile_module_string(code: &str) -> Result<CompiledModule> {
    compile_module_string_and_assert_no_error(code, vec![])
}

#[allow(dead_code)]
pub fn compile_module_string_with_deps(
    code: &str,
    deps: Vec<CompiledModule>,
) -> Result<CompiledModule> {
    compile_module_string_and_assert_no_error(code, deps)
}

#[allow(dead_code)]
pub fn compile_module_string_and_assert_error(
    code: &str,
    deps: Vec<CompiledModule>,
) -> Result<CompiledModule> {
    let (verified_module, verification_errors) = compile_module_string_impl(code, deps)?;
    assert!(!verification_errors.is_empty());
    Ok(verified_module)
}

pub fn count_locals(script: &CompiledScript) -> usize {
    script
        .locals_signature_at(script.main().code.locals)
        .0
        .len()
}

pub fn compile_module_string_with_stdlib(code: &str) -> Result<CompiledModule> {
    compile_module_string_and_assert_no_error(code, build_stdlib(&AccountAddress::default()))
}

pub fn compile_script_string_with_stdlib(code: &str) -> Result<CompiledScript> {
    compile_script_string_and_assert_no_error(code, build_stdlib(&AccountAddress::default()))
}
