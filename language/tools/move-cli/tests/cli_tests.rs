// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use move_cli::sandbox::commands::test;

use std::path::PathBuf;

pub const CLI_BINARY_PATH: [&str; 6] = ["..", "..", "..", "target", "debug", "move"];
pub const CLI_METATEST_PATH: [&str; 3] = ["tests", "metatests", "args.txt"];

fn get_cli_binary_path() -> PathBuf {
    CLI_BINARY_PATH.iter().collect()
}

fn get_metatest_path() -> PathBuf {
    CLI_METATEST_PATH.iter().collect()
}

#[test]
fn run_metatest() {
    let path_cli_binary = get_cli_binary_path();
    let path_metatest = get_metatest_path();

    // local workspace + with coverage
    assert!(test::run_all(&path_metatest, path_cli_binary.as_path(), false, true).is_ok());

    // temp workspace + with coverage
    assert!(test::run_all(&path_metatest, &path_cli_binary, true, true).is_ok());

    // local workspace + without coverage
    assert!(test::run_all(&path_metatest, &path_cli_binary, false, false).is_ok());

    // temp workspace + without coverage
    assert!(test::run_all(&path_metatest, &path_cli_binary, true, false).is_ok());
}
