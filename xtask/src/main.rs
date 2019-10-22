use std::error::Error;
use structopt::StructOpt;

mod test_unit;

#[derive(Debug, StructOpt)]
struct Args {
    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(Debug, StructOpt)]
enum Command {
    #[structopt(name = "test-unit")]
    /// Run unit tests
    TestUnit {
        #[structopt(long, short)]
        /// Restrict unit tests to a specific pacakge
        package: Option<String>,
    },
}

// When invoked as a cargo subcommand, cargo passes too many arguments so we need to filter out
// arg[1] if it matches the end of arg[0], e.i. "cargo-X X foo" should become "cargo-X foo".
fn args() -> impl Iterator<Item = String> {
    let mut args: Vec<String> = ::std::env::args().collect();

    if args.len() >= 2 && args[0].ends_with(&format!("cargo-{}", args[1])) {
        args.remove(1);
    }

    args.into_iter()
}

fn main() -> Result<(), Box<impl Error>> {
    let args = Args::from_iter(args());

    match args.cmd {
        Command::TestUnit { package } => test_unit::run(package),
    }
}
