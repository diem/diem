// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use move_package::BuildConfig;
use structopt::StructOpt;

fn main() {
    let current_dir = std::env::current_dir().unwrap();
    let build_config = BuildConfig::from_args();
    build_config
        .compile_package(&current_dir, &mut std::io::stdout())
        .unwrap();
}
