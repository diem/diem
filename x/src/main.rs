use structopt::StructOpt;

mod cargo;
mod check;
mod clippy;
mod config;
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
    #[structopt(name = "check")]
    /// Run `cargo check`
    Check(check::Args),
    #[structopt(name = "clippy")]
    /// Run `cargo clippy`
    Clippy(clippy::Args),
    #[structopt(name = "test")]
    /// Run tests
    Test(test::Args),
}

fn main() -> Result<()> {
    let args = Args::from_args();
    let config = config::Config::from_project_root()?;

    match args.cmd {
        Command::Test(args) => test::run(args, config),
        Command::Check(args) => check::run(args, config),
        Command::Clippy(args) => clippy::run(args, config),
    }
}
