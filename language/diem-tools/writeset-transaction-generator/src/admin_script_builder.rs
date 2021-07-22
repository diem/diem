// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use diem_types::{
    account_address::AccountAddress,
    account_config::diem_root_address,
    transaction::{Script, WriteSetPayload},
};
use handlebars::Handlebars;
use move_lang::{compiled_unit::AnnotatedCompiledUnit, Compiler, Flags};
use serde::Serialize;
use std::{collections::HashMap, io::Write, path::PathBuf};
use tempfile::NamedTempFile;

/// The relative path to the scripts templates
pub const SCRIPTS_DIR_PATH: &str = "templates";

pub fn compile_script(source_file_str: String) -> Vec<u8> {
    let (_files, mut compiled_program) =
        Compiler::new(&[source_file_str], &diem_framework::diem_stdlib_files())
            .set_flags(Flags::empty().set_sources_shadow_deps(false))
            .set_named_address_values(diem_framework::diem_framework_named_addresses())
            .build_and_report()
            .unwrap();
    assert!(compiled_program.len() == 1);
    match compiled_program.pop().unwrap() {
        AnnotatedCompiledUnit::Module(_) => panic!("Unexpected module when compiling script"),
        x @ AnnotatedCompiledUnit::Script(_) => x.into_compiled_unit().serialize(),
    }
}

fn compile_admin_script(input: &str) -> Result<Script> {
    let mut temp_file = NamedTempFile::new()?;
    temp_file.write_all(input.as_bytes())?;
    let cur_path = temp_file.path().to_str().unwrap().to_owned();
    Ok(Script::new(compile_script(cur_path), vec![], vec![]))
}

pub fn template_path() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push(SCRIPTS_DIR_PATH);
    path
}

pub fn encode_remove_validators_payload(validators: Vec<AccountAddress>) -> WriteSetPayload {
    assert!(!validators.is_empty(), "Unexpected validator set length");
    let mut script = template_path();
    script.push("remove_validators.move");

    let script = {
        let mut hb = Handlebars::new();
        hb.set_strict_mode(true);
        hb.register_template_file("script", script).unwrap();
        let mut data = HashMap::new();
        data.insert("addresses", validators);

        let output = hb.render("script", &data).unwrap();

        compile_admin_script(output.as_str()).unwrap()
    };

    WriteSetPayload::Script {
        script,
        execute_as: diem_root_address(),
    }
}

pub fn encode_custom_script<T: Serialize>(
    script_name_in_templates: &str,
    args: &T,
    execute_as: Option<AccountAddress>,
) -> WriteSetPayload {
    let mut script = template_path();
    script.push(script_name_in_templates);

    let script = {
        let mut hb = Handlebars::new();
        hb.register_template_file("script", script).unwrap();
        hb.set_strict_mode(true);
        let output = hb.render("script", args).unwrap();

        compile_admin_script(output.as_str()).unwrap()
    };

    WriteSetPayload::Script {
        script,
        execute_as: execute_as.unwrap_or_else(diem_root_address),
    }
}

pub fn encode_halt_network_payload() -> WriteSetPayload {
    let mut script = template_path();
    script.push("halt_transactions.move");

    WriteSetPayload::Script {
        script: Script::new(
            compile_script(script.to_str().unwrap().to_owned()),
            vec![],
            vec![],
        ),
        execute_as: diem_root_address(),
    }
}
