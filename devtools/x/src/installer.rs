// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    cargo::Cargo,
    config::{CargoConfig, CargoInstallation},
};
use log::{error, info};
use std::process::Command;

pub struct Installer {
    cargo_config: CargoConfig,
    cargo_installations: Vec<(String, CargoInstallation)>,
}

impl Installer {
    pub fn new(
        cargo_config: CargoConfig,
        cargo_installations: Vec<(String, CargoInstallation)>,
    ) -> Installer {
        Self {
            cargo_config,
            cargo_installations,
        }
    }

    // Given the name of a tool in the tools list, return the version.
    fn cargo_installation(&self, name: &str) -> Option<&CargoInstallation> {
        self.cargo_installations
            .as_slice()
            .iter()
            .find_map(|(x, y)| if x.eq(&name) { Some(y) } else { None })
    }

    pub fn install_if_needed(&self, name: &str) -> bool {
        match &self.cargo_installation(name) {
            Some(cargo_installation) => {
                install_if_needed(&self.cargo_config, name, &cargo_installation)
            }
            None => {
                info!("No version of tool {} is specified ", name);
                false
            }
        }
    }

    #[allow(dead_code)]
    fn check(&self, name: &str) -> bool {
        match &self.cargo_installation(name) {
            Some(cargo_installation) => check_installed(name, &cargo_installation.version),
            None => {
                info!("No version of tool {} is specified ", name);
                false
            }
        }
    }

    pub fn check_all(&self) -> bool {
        let iter = self
            .cargo_installations
            .as_slice()
            .iter()
            .map(|(name, installation)| (name, &installation.version))
            .collect::<Vec<(&String, &String)>>();
        check_all(iter.as_slice())
    }

    pub fn install_all(&self) -> bool {
        install_all(&self.cargo_config, &self.cargo_installations.as_slice())
    }
}

pub fn install_if_needed(
    cargo_config: &CargoConfig,
    name: &str,
    installation: &CargoInstallation,
) -> bool {
    if !check_installed(name, &installation.version) {
        info!("Installing {} {}", name, installation.version);
        //prevent recursive install attempts of sccache.
        let mut cmd = Cargo::new(&cargo_config, "install", false);
        if let Some(features) = &installation.features {
            if !features.is_empty() {
                cmd.arg("--features");
                cmd.args(features);
            }
        }
        if let Some(git_url) = &installation.git {
            cmd.arg("--git");
            cmd.arg(git_url);
            if let Some(git_rev) = &installation.git_rev {
                cmd.arg("--rev");
                cmd.arg(git_rev);
            }
        } else {
            cmd.arg("--version").arg(&installation.version);
        }
        cmd.arg(name);

        let result = cmd.run();
        if result.is_err() {
            error!(
                "Could not install {} {}, check x.toml to ensure tool exists and is not yanked, or provide a git-rev if your x.toml specifies a git-url.",
                name, installation.version
            );
        }
        result.is_ok()
    } else {
        true
    }
}

//TODO check installed features for sccache?
fn check_installed(name: &str, version: &str) -> bool {
    let result = Command::new(name).arg("--version").output();
    let found = match result {
        Ok(output) => format!("{} {}", name, version)
            .eq(String::from_utf8_lossy(&output.stdout.as_slice()).trim()),
        _ => false,
    };
    info!(
        "{} of version {} is{} installed",
        name,
        version,
        if !found { " not" } else { "" }
    );
    found
}

fn install_all(config: &CargoConfig, tools: &[(String, CargoInstallation)]) -> bool {
    let mut success: bool = true;
    for (name, installation) in tools {
        success &= install_if_needed(config, name, installation);
    }
    success
}

fn check_all(tools: &[(&String, &String)]) -> bool {
    let mut success: bool = true;
    for (key, value) in tools {
        success &= check_installed(key, value);
    }
    success
}
