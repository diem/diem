// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use move_cli::test;

use std::path::PathBuf;

pub const CLI_BINARY_PATH: [&str; 6] = ["..", "..", "..", "target", "debug", "move-cli"];
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
    assert!(test::run_all(&get_metatest_path(), &get_cli_binary_path()).is_ok());
}
