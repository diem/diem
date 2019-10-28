use crate::{
    cargo::Cargo,
    config::{Config, Package},
    utils::{self, project_root},
    Result,
};
use std::ffi::{OsStr, OsString};
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
        run_cargo_test_on_packages_separate(
            config
                .package_exceptions()
                .iter()
                .filter(|(_, pkg)| !pkg.system),
            &args.args,
        )?;
        run_cargo_test_with_exclusions(
            config.package_exceptions().iter().map(|(p, _)| p),
            &args.args,
        )?;
        Ok(())
    } else if !args.package.is_empty() {
        let run_together = args.package.iter().filter(|p| !config.is_exception(p));
        let run_separate = args
            .package
            .iter()
            .filter_map(|p| config.package_exceptions().get(p).map(|e| (p, e)));
        run_cargo_test_on_packages_separate(run_separate, &args.args)?;
        run_cargo_test_on_packages_together(run_together, &args.args)?;
        Ok(())
    } else if utils::project_is_root()? {
        // TODO Maybe only run a subest of tests if we're not inside
        // a package but not at the project root (e.g. language)
        run_cargo_test_on_packages_separate(config.package_exceptions(), &args.args)?;
        run_cargo_test_with_exclusions(
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

        run_cargo_test_on_local_package(all_features, &args.args)?;
        Ok(())
    }
}

fn run_cargo_test_on_local_package(
    all_features: bool,
    pass_through_args: &[OsString],
) -> Result<()> {
    let mut cargo = Cargo::new("test");
    if all_features {
        cargo.all_features();
    }
    cargo.pass_through(pass_through_args).run()
}

fn run_cargo_test_on_packages_separate<'a, I, S>(
    packages: I,
    pass_through_args: &[OsString],
) -> Result<()>
where
    I: IntoIterator<Item = (S, &'a Package)>,
    S: AsRef<OsStr>,
{
    for (name, pkg) in packages {
        let mut cargo = Cargo::new("test");
        if pkg.all_features {
            cargo.all_features();
        }
        cargo
            .packages(&[name])
            .current_dir(project_root())
            .pass_through(pass_through_args)
            .run()?;
    }
    Ok(())
}

fn run_cargo_test_on_packages_together<I, S>(
    packages: I,
    pass_through_args: &[OsString],
) -> Result<()>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    Cargo::new("test")
        .current_dir(project_root())
        .all_features()
        .packages(packages)
        .pass_through(pass_through_args)
        .run()
}

fn run_cargo_test_with_exclusions<I, S>(exclusions: I, pass_through_args: &[OsString]) -> Result<()>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    Cargo::new("test")
        .current_dir(project_root())
        .all()
        .all_features()
        .exclusions(exclusions)
        .pass_through(pass_through_args)
        .run()
}
