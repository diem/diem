// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    cargo::Cargo,
    config::{CargoConfig, CargoInstallation},
};
use log::{error, info, warn};
use std::process::Command;

pub struct Installer {
    cargo_config: CargoConfig,
    cargo_installations: Vec<(String, CargoInstallation)>,
    rust_installation: Vec<String>,
}

impl Installer {
    pub fn new(
        cargo_config: CargoConfig,
        cargo_installations: Vec<(String, CargoInstallation)>,
        rust_installation: Vec<String>,
    ) -> Installer {
        Self {
            cargo_config,
            cargo_installations,
            rust_installation,
        }
    }

    // Given the name of a tool in the tools list, return the version.
    fn cargo_installation(&self, name: &str) -> Option<&CargoInstallation> {
        self.cargo_installations
            .as_slice()
            .iter()
            .find_map(|(x, y)| if x.eq(&name) { Some(y) } else { None })
    }

    pub fn install_via_cargo_if_needed(&self, name: &str) -> bool {
        match &self.cargo_installation(name) {
            Some(cargo_installation) => {
                install_cargo_component_if_needed(&self.cargo_config, name, &cargo_installation)
            }
            None => {
                info!("No version of tool {} is specified ", name);
                false
            }
        }
    }

    pub fn install_via_rustup_if_needed(&self, name: &str) -> bool {
        install_rustup_component_if_needed(name)
    }

    #[allow(dead_code)]
    fn check_cargo_component(&self, name: &str) -> bool {
        match &self.cargo_installation(name) {
            Some(cargo_installation) => {
                check_installed_cargo_component(name, &cargo_installation.version)
            }
            None => {
                info!("No version of tool {} is specified ", name);
                false
            }
        }
    }

    #[allow(dead_code)]
    fn install_rustup_component(&self, name: &str) -> bool {
        install_rustup_component_if_needed(name)
    }

    pub fn check_all(&self) -> bool {
        let iter = self
            .cargo_installations
            .as_slice()
            .iter()
            .map(|(name, installation)| (name, &installation.version))
            .collect::<Vec<(&String, &String)>>();
        check_all_cargo_components(iter.as_slice())
    }

    pub fn install_all(&self) -> bool {
        let mut result =
            install_all_cargo_components(&self.cargo_config, &self.cargo_installations.as_slice());
        result &= install_all_rustup_components(&self.rust_installation.as_slice());
        result
    }
}

pub fn install_rustup_component_if_needed(name: &str) -> bool {
    let mut cmd = Command::new("rustup");
    cmd.args(&["component", "list", "--installed"]);
    let result = cmd.output();

    let installed = if let Ok(output) = result {
        let bytes = output.stdout.as_slice();
        let installed_components = String::from_utf8_lossy(bytes);
        installed_components.contains(name)
    } else {
        false
    };
    if !installed {
        info!("installing rustup component: {}", name);
        let mut cmd = Command::new("rustup");
        cmd.args(&["component", "add", name]);
        if cmd.output().is_ok() {
            info!("rustup component {} has been installed", name);
        } else {
            warn!("rustup component {} failed to install", name);
        }
        cmd.output().is_ok()
    } else {
        info!("rustup component {} is already installed", name);
        true
    }
}

fn install_all_rustup_components(names: &[String]) -> bool {
    let mut result = true;
    for name in names {
        result &= install_rustup_component_if_needed(name);
    }
    result
}

pub fn install_cargo_component_if_needed(
    cargo_config: &CargoConfig,
    name: &str,
    installation: &CargoInstallation,
) -> bool {
    if !check_installed_cargo_component(name, &installation.version) {
        info!("Installing {} {}", name, installation.version);
        //prevent recursive install attempts of sccache.
        let mut cmd = Cargo::new(&cargo_config, "install", true);
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
fn check_installed_cargo_component(name: &str, version: &str) -> bool {
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

fn install_all_cargo_components(
    config: &CargoConfig,
    tools: &[(String, CargoInstallation)],
) -> bool {
    let mut success: bool = true;
    for (name, installation) in tools {
        success &= install_cargo_component_if_needed(config, name, installation);
    }
    success
}

fn check_all_cargo_components(tools: &[(&String, &String)]) -> bool {
    let mut success: bool = true;
    for (key, value) in tools {
        success &= check_installed_cargo_component(key, value);
    }
    success
}
