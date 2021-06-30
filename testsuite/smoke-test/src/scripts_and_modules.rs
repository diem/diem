// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    test_utils::{compare_balances, setup_swarm_and_client_proxy},
    workspace_builder,
};
use diem_temppath::TempPath;
use diem_types::account_address::AccountAddress;
use move_move_command_line_common::files::MOVE_EXTENSION;
use std::{
    fs, io,
    io::Write,
    path::{Path, PathBuf},
};

#[test]
fn test_malformed_script() {
    let (_env, mut client) = setup_swarm_and_client_proxy(1, 0);
    client
        .enable_custom_script(&["enable_custom_script"], false, true)
        .unwrap();
    client.create_next_account(false).unwrap();
    client
        .mint_coins(&["mintb", "0", "100", "XUS"], true)
        .unwrap();

    let script_path = workspace_builder::workspace_root()
        .join("testsuite/smoke-test/src/dev_modules/test_script.move");

    let unwrapped_script_path = script_path.to_str().unwrap();
    let move_stdlib_dir = move_stdlib::move_stdlib_modules_full_path();
    let diem_framework_dir = diem_framework::diem_stdlib_modules_full_path();
    let script_params = &[
        "compile",
        unwrapped_script_path,
        move_stdlib_dir.as_str(),
        diem_framework_dir.as_str(),
    ];
    let mut script_compiled_paths = client.compile_program(script_params).unwrap();
    let script_compiled_path = if script_compiled_paths.len() != 1 {
        panic!("compiler output has more than one file")
    } else {
        script_compiled_paths.pop().unwrap()
    };

    // the script expects two arguments. Passing only one in the test, which will cause a failure.
    client
        .execute_script(&["execute", "0", &script_compiled_path[..], "10"])
        .expect_err("malformed script did not fail!");

    // Previous transaction should not choke the system.
    client
        .mint_coins(&["mintb", "0", "10", "XUS"], true)
        .unwrap();
}

#[test]
fn test_execute_custom_module_and_script() {
    let (_env, mut client) = setup_swarm_and_client_proxy(1, 0);
    client
        .enable_custom_script(&["enable_custom_script"], true, true)
        .unwrap();
    client.create_next_account(false).unwrap();
    client
        .mint_coins(&["mintb", "0", "50", "XUS"], true)
        .unwrap();
    assert!(compare_balances(
        vec![(50.0, "XUS".to_string())],
        client.get_balances(&["b", "0"]).unwrap(),
    ));

    let recipient_address = client.create_next_account(false).unwrap().address;
    client
        .mint_coins(&["mintb", "1", "1", "XUS"], true)
        .unwrap();

    let (sender_account, _) = client.get_account_address_from_parameter("0").unwrap();

    // Get the path to the Move stdlib sources
    let move_stdlib_dir = move_stdlib::move_stdlib_modules_full_path();
    let diem_framework_dir = diem_framework::diem_stdlib_modules_full_path();

    // Make a copy of module.move with "{{sender}}" substituted.
    let module_path = workspace_builder::workspace_root()
        .join("testsuite/smoke-test/src/dev_modules/module.move");
    let copied_module_path = copy_file_with_sender_address(&module_path, sender_account).unwrap();
    let unwrapped_module_path = copied_module_path.to_str().unwrap();

    // Compile and publish that module.
    let module_params = &[
        "compile",
        unwrapped_module_path,
        move_stdlib_dir.as_str(),
        diem_framework_dir.as_str(),
    ];
    let mut module_compiled_paths = client.compile_program(module_params).unwrap();
    let module_compiled_path = if module_compiled_paths.len() != 1 {
        panic!("compiler output has more than one file")
    } else {
        module_compiled_paths.pop().unwrap()
    };
    client
        .publish_module(&["publish", "0", &module_compiled_path[..]])
        .unwrap();

    // Make a copy of script.move with "{{sender}}" substituted.
    let script_path = workspace_builder::workspace_root()
        .join("testsuite/smoke-test/src/dev_modules/script.move");
    let copied_script_path = copy_file_with_sender_address(&script_path, sender_account).unwrap();
    let unwrapped_script_path = copied_script_path.to_str().unwrap();

    // Compile and execute the script.
    let script_params = &[
        "compile",
        unwrapped_script_path,
        unwrapped_module_path,
        move_stdlib_dir.as_str(),
        diem_framework_dir.as_str(),
    ];
    let mut script_compiled_paths = client.compile_program(script_params).unwrap();
    let script_compiled_path = if script_compiled_paths.len() != 1 {
        panic!("compiler output has more than one file")
    } else {
        script_compiled_paths.pop().unwrap()
    };
    let formatted_recipient_address = format!("0x{}", recipient_address);
    client
        .execute_script(&[
            "execute",
            "0",
            &script_compiled_path[..],
            &formatted_recipient_address[..],
            "10",
        ])
        .unwrap();

    assert!(compare_balances(
        vec![(49.999_990, "XUS".to_string())],
        client.get_balances(&["b", "0"]).unwrap(),
    ));
    assert!(compare_balances(
        vec![(1.000_010, "XUS".to_string())],
        client.get_balances(&["b", "1"]).unwrap(),
    ));
}

fn copy_file_with_sender_address(file_path: &Path, sender: AccountAddress) -> io::Result<PathBuf> {
    let tmp_source_path = TempPath::new().as_ref().with_extension(MOVE_EXTENSION);
    let mut tmp_source_file = std::fs::File::create(tmp_source_path.clone())?;
    let mut code = fs::read_to_string(file_path)?;
    code = code.replace("{{sender}}", &format!("0x{}", sender));
    writeln!(tmp_source_file, "{}", code)?;
    Ok(tmp_source_path)
}
