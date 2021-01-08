// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use std::path::Path;
use structopt::StructOpt;

use move_visibility::pkg_stdlib::PackageStdlib;

#[derive(StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct Args {
    /// Package to analyze
    #[structopt(short, long, default_value = "stdlib")]
    package: String,
}

fn main() -> Result<()> {
    let args = Args::from_args();

    let pkg = if args.package == "stdlib" {
        PackageStdlib::new()
    } else {
        panic!("Invalid package name")
    };

    pkg.run(Path::new("."))
}
