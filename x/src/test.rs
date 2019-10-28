use crate::{cargo::CargoCommand, config::Config, utils, Result};
use std::ffi::OsString;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Args {
    #[structopt(long, short, number_of_values = 1)]
    /// Run test on the provided packages
    package: Vec<String>,
    #[structopt(long, short)]
    /// Only run unit tests
    unit: bool,
    #[structopt(name = "TESTNAME", parse(from_os_str))]
    testname: Option<OsString>,
    #[structopt(name = "ARGS", parse(from_os_str), last = true)]
    args: Vec<OsString>,
}

pub fn run(mut args: Args, config: Config) -> Result<()> {
    args.args.extend(args.testname.clone());

    if args.unit {
        CargoCommand::Test.run_on_packages_separate(
            config
                .package_exceptions()
                .iter()
                .filter(|(_, pkg)| !pkg.system)
                .map(|(name, pkg)| (name, pkg.all_features)),
            &args.args,
        )?;
        CargoCommand::Test.run_with_exclusions(
            config.package_exceptions().iter().map(|(p, _)| p),
            &args.args,
        )?;
        Ok(())
    } else if !args.package.is_empty() {
        let run_together = args.package.iter().filter(|p| !config.is_exception(p));
        let run_separate = args.package.iter().filter_map(|p| {
            config
                .package_exceptions()
                .get(p)
                .map(|e| (p, e.all_features))
        });
        CargoCommand::Test.run_on_packages_separate(run_separate, &args.args)?;
        CargoCommand::Test.run_on_packages_together(run_together, &args.args)?;
        Ok(())
    } else if utils::project_is_root()? {
        // TODO Maybe only run a subest of tests if we're not inside
        // a package but not at the project root (e.g. language)
        CargoCommand::Test.run_on_packages_separate(
            config
                .package_exceptions()
                .iter()
                .map(|(name, pkg)| (name, pkg.all_features)),
            &args.args,
        )?;
        CargoCommand::Test.run_with_exclusions(
            config.package_exceptions().iter().map(|(p, _)| p),
            &args.args,
        )?;
        Ok(())
    } else {
        let package = utils::get_local_package()?;
        let all_features = config
            .package_exceptions()
            .get(&package)
            .map(|pkg| pkg.all_features)
            .unwrap_or(true);

        CargoCommand::Test.run_on_local_package(all_features, &args.args)?;
        Ok(())
    }
}
