// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{context::XContext, installer, Result};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Args {
    #[structopt(long)]
    /// Run in 'check' mode. Exits with 0 if all tools installed. Exits with 1 and if not, printing failed
    check: bool,
}

pub fn run(args: Args, xctx: XContext) -> Result<()> {
    let tools = xctx.config().tools();
    let mut failed: bool = false;
    for (key, value) in tools {
        let success = match args.check {
            false => installer::install_if_needed(key, value),
            true => installer::check_installed(key, value),
        };
        failed = failed || !success;
    }
    if failed {
        std::process::exit(1);
    }
    Ok(())
}
