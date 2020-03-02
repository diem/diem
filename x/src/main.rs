// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use env_logger::Env;
use structopt::StructOpt;

mod bench;
mod cargo;
mod check;
mod clippy;
mod config;
mod fmt;
mod lint;
mod test;
mod utils;

type Result<T> = anyhow::Result<T>;

#[derive(Debug, StructOpt)]
struct Args {
    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(Debug, StructOpt)]
enum Command {
    #[structopt(name = "bench")]
    /// Run `cargo bench`
    Bench(bench::Args),
    #[structopt(name = "check")]
    /// Run `cargo check`
    Check(check::Args),
    #[structopt(name = "clippy")]
    /// Run `cargo clippy`
    Clippy(clippy::Args),
    #[structopt(name = "fmt")]
    /// Run `cargo fmt`
    Fmt(fmt::Args),
    #[structopt(name = "test")]
    /// Run tests
    Test(test::Args),
    #[structopt(name = "lint")]
    /// Run lints
    Lint(lint::Args),
}

fn main() -> Result<()> {
    let env = Env::default()
        .filter_or("MY_LOG_LEVEL", "info")
        .write_style_or("MY_LOG_STYLE", "always");
    env_logger::init_from_env(env);

    let args = Args::from_args();
    let config = config::Config::from_project_root()?;

    match args.cmd {
        Command::Test(args) => test::run(args, config),
        Command::Check(args) => check::run(args, config),
        Command::Clippy(args) => clippy::run(args, config),
        Command::Fmt(args) => fmt::run(args, config),
        Command::Bench(args) => bench::run(args, config),
        Command::Lint(args) => lint::run(args, config),
    }
}
