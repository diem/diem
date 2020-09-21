// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{cargo::Cargo, config::CargoConfig};
use log::info;
use std::process::Command;

pub fn install_if_needed(name: &str, version: &str) -> bool {
    //This is a bit of a hack, given Cargo wants the CargoConfig struct, but we shouldn't be using the nightly
    //cargo's tooling to build our 3rd party cargo tools.  Looking to address that in a refactoring pr once all the needed
    //components are in place -- perhaps by seperating xcontext in to constituent parts.
    let config = CargoConfig {
        toolchain: "stable".to_owned(),
        flags: Option::None,
    };
    install_if_needed_config(&config, name, version)
}

pub fn install_if_needed_config(config: &CargoConfig, name: &str, version: &str) -> bool {
    if !check_installed(name, version) {
        info!("Installing {} {}", name, version);
        let installers = Cargo::new(config, "install")
            .arg(name)
            .arg("--version")
            .arg(version)
            .run();
        if installers.is_err() {
            info!(
                "Could not install {} {}, check x.toml to ensure tool exists and is not yanked.",
                name, version
            );
        }
        installers.is_ok()
    } else {
        true
    }
}

pub fn check_installed(name: &str, version: &str) -> bool {
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
