// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    test_utils::{compare_balances, setup_swarm_and_client_proxy},
    workspace_builder,
};
use diem_crypto::HashValue;
use diem_temppath::TempPath;
use diem_types::account_address::AccountAddress;
use std::{
    fs, io,
    io::Write,
    path::{Path, PathBuf},
};

#[test]
fn test_e2e_modify_publishing_option() {
    let (_env, mut client) = setup_swarm_and_client_proxy(1, 0);
    client.create_next_account(false).unwrap();

    client
        .mint_coins(&["mintb", "0", "10", "XUS"], true)
        .unwrap();
    assert!(compare_balances(
        vec![(10.0, "XUS".to_string())],
        client.get_balances(&["b", "0"]).unwrap(),
    ));
    let script_path = workspace_builder::workspace_root()
        .join("testsuite/smoke-test/src/dev_modules/test_script.move");
    let unwrapped_script_path = script_path.to_str().unwrap();
    let stdlib_source_dir = workspace_builder::workspace_root().join("language/stdlib/modules");
    let unwrapped_stdlib_dir = stdlib_source_dir.to_str().unwrap();
    let script_params = &["compile", "0", unwrapped_script_path, unwrapped_stdlib_dir];
    let mut script_compiled_paths = client.compile_program(script_params).unwrap();
    let script_compiled_path = if script_compiled_paths.len() != 1 {
        panic!("compiler output has more than one file")
    } else {
        script_compiled_paths.pop().unwrap()
    };

    // Initially publishing option was set to CustomScript, this transaction should be executed.
    client
        .execute_script(&["execute", "0", &script_compiled_path[..], "10", "0x0"])
        .unwrap();

    // Make sure the transaction is executed by checking if the sequence is bumped to 1.
    assert_eq!(
        client
            .get_sequence_number(&["sequence", "0", "true"])
            .unwrap(),
        1
    );

    let hash = hex::encode(&HashValue::random().to_vec());

    client
        .add_to_script_allow_list(&["add_to_script_allow_list", hash.as_str()], true)
        .unwrap();

    // Now that publishing option was changed to locked, this transaction will be rejected.
    assert!(format!(
        "{:?}",
        client
            .execute_script(&["execute", "0", &script_compiled_path[..], "10", "0x0"])
            .unwrap_err()
            .root_cause()
    )
    .contains("UNKNOWN_SCRIPT"));

    assert_eq!(
        client
            .get_sequence_number(&["sequence", "0", "true"])
            .unwrap(),
        1
    );
}

#[test]
fn test_malformed_script() {
    let (_env, mut client) = setup_swarm_and_client_proxy(1, 0);
    client.create_next_account(false).unwrap();
    client
        .mint_coins(&["mintb", "0", "100", "XUS"], true)
        .unwrap();

    let script_path = workspace_builder::workspace_root()
        .join("testsuite/smoke-test/src/dev_modules/test_script.move");
    let unwrapped_script_path = script_path.to_str().unwrap();
    let stdlib_source_dir = workspace_builder::workspace_root().join("language/stdlib/modules");
    let unwrapped_stdlib_dir = stdlib_source_dir.to_str().unwrap();
    let script_params = &["compile", "0", unwrapped_script_path, unwrapped_stdlib_dir];
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
    let stdlib_source_dir = workspace_builder::workspace_root().join("language/stdlib/modules");
    let unwrapped_stdlib_dir = stdlib_source_dir.to_str().unwrap();

    // Make a copy of module.move with "{{sender}}" substituted.
    let module_path = workspace_builder::workspace_root()
        .join("testsuite/smoke-test/src/dev_modules/module.move");
    let copied_module_path = copy_file_with_sender_address(&module_path, sender_account).unwrap();
    let unwrapped_module_path = copied_module_path.to_str().unwrap();

    // Compile and publish that module.
    let module_params = &["compile", "0", unwrapped_module_path, unwrapped_stdlib_dir];
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
        "0",
        unwrapped_script_path,
        unwrapped_module_path,
        unwrapped_stdlib_dir,
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
    let tmp_source_path = TempPath::new().as_ref().with_extension("move");
    let mut tmp_source_file = std::fs::File::create(tmp_source_path.clone())?;
    let mut code = fs::read_to_string(file_path)?;
    code = code.replace("{{sender}}", &format!("0x{}", sender));
    writeln!(tmp_source_file, "{}", code)?;
    Ok(tmp_source_path)
}
