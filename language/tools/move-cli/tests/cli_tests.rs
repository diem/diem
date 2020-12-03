// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use move_cli::test;

use std::path::PathBuf;

pub const CLI_BINARY_PATH: [&str; 6] = ["..", "..", "..", "target", "debug", "move"];
pub const CLI_METATEST_PATH: [&str; 3] = ["tests", "metatests", "args.txt"];

fn get_cli_binary_path() -> String {
    let pb: PathBuf = CLI_BINARY_PATH.iter().collect();
    pb.to_str().unwrap().to_owned()
}

fn get_metatest_path() -> String {
    let pb: PathBuf = CLI_METATEST_PATH.iter().collect();
    pb.to_str().unwrap().to_owned()
}

#[test]
fn run_metatest() {
    let path_cli_binary = get_cli_binary_path();
    let path_metatest = get_metatest_path();

    // with coverage
    assert!(test::run_all(&path_metatest, &path_cli_binary, true).is_ok());
    // without coverage
    assert!(test::run_all(&path_metatest, &path_cli_binary, false).is_ok());
}
