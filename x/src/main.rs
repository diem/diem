use structopt::StructOpt;

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
    #[structopt(name = "test")]
    /// Run tests
    Test(test::Args),
}

fn main() -> Result<()> {
    let args = Args::from_args();
    let config = config::Config::from_project_root()?;

    match args.cmd {
        Command::Test(args) => test::run(args, config),
    }
}
