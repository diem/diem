// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anchor;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(rename_all = "kebab-case", about = "The Anchor of Libra \u{2693}")]
struct Args {
    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(Debug, StructOpt)]
enum Command {
    #[structopt(name = "lint")]
    /// Perform lint operations
    Lint,
    #[structopt(name = "setup")]
    /// Check and setup the developer environment
    Setup,
}

fn main() {
    let args = Args::from_args();

    let result = match args.cmd {
        Command::Lint => Ok(()),
        Command::Setup => anchor::setup::exec(),
    };

    match result {
        Err(e) => println!("{}\nAborting...", e),
        Ok(()) => {}
    }
}
