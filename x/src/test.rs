use crate::config::Package;
use crate::{
    config::Config,
    utils::{self, project_root},
    Result,
};
use anyhow::anyhow;
use std::iter;
use std::process::{Command, Stdio};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Args {
    #[structopt(long, short, number_of_values = 1)]
    /// Run test on the provided packages
    package: Vec<String>,
    #[structopt(long, short)]
    /// Only run unit tests
    unit: bool,
}

pub fn run(args: Args, config: Config) -> Result<()> {
    if args.unit {
        run_cargo_test_on_packages_separate(
            config
                .package_exceptions()
                .iter()
                .filter(|(_, pkg)| !pkg.system)
                .map(|(p, pkg)| (p.as_str(), pkg)),
        )?;
        run_cargo_test_with_exclusions(
            config.package_exceptions().iter().map(|(p, _)| p.as_str()),
        )?;
        Ok(())
    } else if !args.package.is_empty() {
        let run_together = args.package.iter().filter(|p| !config.is_exception(p));
        let run_separate = args
            .package
            .iter()
            .filter(|p| config.is_exception(p))
            .map(|p| (p.as_str(), config.package_exceptions().get(p).unwrap()));
        run_cargo_test_on_packages_separate(run_separate)?;
        run_cargo_test_on_packages_together(run_together.map(|s| s.as_str()))?;
        Ok(())
    } else if utils::project_is_root()? {
        // TODO Maybe only run a subest of tests if we're not inside
        // a package but not at the project root (e.g. language)
        run_cargo_test_on_packages_separate(
            config
                .package_exceptions()
                .iter()
                .map(|(p, pkg)| (p.as_str(), pkg)),
        )?;
        run_cargo_test_with_exclusions(
            config.package_exceptions().iter().map(|(p, _)| p.as_str()),
        )?;
        Ok(())
    } else {
        let package = utils::get_local_package()?;
        let all_features = config
            .package_exceptions()
            .get(&package)
            .map(|pkg| pkg.all_features)
            .unwrap_or(true);

        run_cargo_test_on_local_package(all_features)?;
        Ok(())
    }
}

fn run_cargo_test_on_local_package(all_features: bool) -> Result<()> {
    let args = if all_features {
        vec!["test", "--all-features"]
    } else {
        vec!["test"]
    };
    let output = Command::new("cargo")
        .args(args)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()?;
    if !output.status.success() {
        return Err(anyhow!("failed to test local package"));
    }
    Ok(())
}

fn run_cargo_test_on_packages_separate<'a>(
    packages: impl Iterator<Item = (&'a str, &'a Package)>,
) -> Result<()> {
    for (name, pkg) in packages {
        let mut args = if pkg.all_features {
            vec!["test", "--all-features"]
        } else {
            vec!["test"]
        };
        args.push("-p");
        args.push(name);
        let output = Command::new("cargo")
            .current_dir(project_root())
            .args(args)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output()?;
        if !output.status.success() {
            return Err(anyhow!("failed to test package {}", name));
        }
    }
    Ok(())
}

fn run_cargo_test_on_packages_together<'a>(packages: impl Iterator<Item = &'a str>) -> Result<()> {
    let output = Command::new("cargo")
        .current_dir(project_root())
        .args(&["test", "--all-features"])
        .args(
            iter::repeat("-p")
                .zip(packages)
                .flat_map(|tup| iter::once(tup.0).chain(iter::once(tup.1))),
        )
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()?;
    if !output.status.success() {
        return Err(anyhow!("failed to test packages"));
    }
    Ok(())
}

fn run_cargo_test_with_exclusions<'a>(exclude: impl Iterator<Item = &'a str>) -> Result<()> {
    let output = Command::new("cargo")
        .current_dir(project_root())
        .args(&["test", "--all", "--all-features"])
        .args(
            iter::repeat("--exclude")
                .zip(exclude)
                .flat_map(|tup| iter::once(tup.0).chain(iter::once(tup.1))),
        )
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()?;
    if !output.status.success() {
        return Err(anyhow!("failed to test packages"));
    }
    Ok(())
}
