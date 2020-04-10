// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use bytecode_verifier::{VerifiedModule, VerifiedScript};
use ir_to_bytecode::{
    compiler::{compile_module, compile_script},
    parser::{parse_module, parse_script},
};
use libra_types::{account_address::AccountAddress, vm_error::VMStatus};
use stdlib::stdlib_modules;
use vm::{
    access::ScriptAccess,
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
) -> Result<(CompiledScript, Vec<VMStatus>)> {
    let parsed_script = parse_script("file_name", code).unwrap();
    let compiled_script = compile_script(AccountAddress::default(), parsed_script, &deps)?.0;

    let mut serialized_script = Vec::<u8>::new();
    compiled_script.serialize(&mut serialized_script)?;
    let deserialized_script = CompiledScript::deserialize(&serialized_script)?;
    assert_eq!(compiled_script, deserialized_script);

    // Always return a CompiledScript because some callers explicitly care about unverified
    // modules.
    let ret = match VerifiedScript::new(compiled_script) {
        Ok(script) => (script.into_inner(), vec![]),
        Err((script, errors)) => (script, errors),
    };
    Ok(ret)
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
) -> Result<(CompiledModule, Vec<VMStatus>)> {
    let address = AccountAddress::default();
    let module = parse_module("file_name", code).unwrap();
    let compiled_module = compile_module(address, module, &deps)?.0;

    let mut serialized_module = Vec::<u8>::new();
    compiled_module.serialize(&mut serialized_module)?;
    let deserialized_module = CompiledModule::deserialize(&serialized_module)?;
    assert_eq!(compiled_module, deserialized_module);

    // Always return a CompiledModule because some callers explicitly care about unverified
    // modules.
    let ret = match VerifiedModule::new(compiled_module) {
        Ok(module) => (module.into_inner(), vec![]),
        Err((module, errors)) => (module, errors),
    };
    Ok(ret)
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
    script.signature_at(script.main().code.locals).0.len()
}

pub fn compile_module_string_with_stdlib(code: &str) -> Result<CompiledModule> {
    compile_module_string_and_assert_no_error(code, stdlib())
}

pub fn compile_script_string_with_stdlib(code: &str) -> Result<CompiledScript> {
    compile_script_string_and_assert_no_error(code, stdlib())
}

fn stdlib() -> Vec<CompiledModule> {
    stdlib_modules()
        .iter()
        .map(|m| m.clone().into_inner())
        .collect()
}
