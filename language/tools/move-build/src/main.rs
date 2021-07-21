// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use move_build::BuildConfig;
use structopt::StructOpt;

fn main() {
    let current_dir = std::env::current_dir().unwrap();
    let build_config = BuildConfig::from_args();
    build_config.build(&current_dir).unwrap();
}
