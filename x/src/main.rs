// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use chrono::Local;
use env_logger::{self, fmt::Color};
use log::Level;
use std::io::Write;
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
    env_logger::from_env(env_logger::Env::default().default_filter_or("info"))
        .format(|buf, record| {
            let color = match record.level() {
                Level::Warn => Color::Yellow,
                Level::Error => Color::Red,
                _ => Color::Green,
            };

            let mut level_style = buf.style();
            level_style.set_color(color).set_bold(true);

            writeln!(
                buf,
                "{:>12} [{}] - {}",
                level_style.value(record.level()),
                Local::now().format("%T%.3f"),
                record.args()
            )
        })
        .init();

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
