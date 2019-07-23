// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use failure::bail;
use std::{io::Write, path::Path, process::Command};

enum PackageManager {
    AptGet,
    Pacman,
    Yum,
    Brew,
}

impl PackageManager {
    fn get_system_package_manager() -> Result<PackageManager, failure::Error> {
        let pkg_mgr = if cfg!(target_os = "linux") {
            if is_bin_on_path("apt-get")? {
                PackageManager::AptGet
            } else if is_bin_on_path("pacman")? {
                PackageManager::Pacman
            } else if is_bin_on_path("yum")? {
                PackageManager::Yum
            } else {
                bail!("Unable to find supported package manager (yum, apt-get, or pacman)");
            }
        } else if cfg!(target_os = "macos") {
            if is_bin_on_path("brew")? {
                PackageManager::Brew
            } else {
                bail!("Missing package manager Homebrew (https://brew.sh/)");
            }
        } else {
            bail!("Unknown OS");
        };

        Ok(pkg_mgr)
    }

    fn apt_get<P: AsRef<Path>>(bin: P) -> Result<(), failure::Error> {
        let status = Command::new("sudo")
            .arg("apt-get")
            .arg("install")
            .arg(bin.as_ref())
            .arg("-y")
            .status()?;

        if !status.success() {
            bail!("Running apt-get failed!");
        }

        Ok(())
    }

    fn pacman<P: AsRef<Path>>(bin: P) -> Result<(), failure::Error> {
        let status = Command::new("sudo")
            .arg("pacman")
            .arg("-Syu")
            .arg(bin.as_ref())
            .arg("--noconfirm")
            .status()?;

        if !status.success() {
            bail!("Running pacman failed!");
        }

        Ok(())
    }

    fn yum<P: AsRef<Path>>(bin: P) -> Result<(), failure::Error> {
        let status = Command::new("sudo")
            .arg("yum")
            .arg("install")
            .arg(bin.as_ref())
            .arg("-y")
            .status()?;

        if !status.success() {
            bail!("Running yum failed!");
        }

        Ok(())
    }

    fn brew<P: AsRef<Path>>(bin: P) -> Result<(), failure::Error> {
        let status = Command::new("brew")
            .arg("install")
            .arg(bin.as_ref())
            .status()?;

        if !status.success() {
            bail!("Running brew failed!");
        }

        Ok(())
    }

    fn install_binary<P: AsRef<Path>>(&self, bin: P) -> Result<(), failure::Error> {
        match self {
            PackageManager::AptGet => Self::apt_get(bin),
            PackageManager::Pacman => Self::pacman(bin),
            PackageManager::Yum => Self::yum(bin),
            PackageManager::Brew => Self::brew(bin),
        }
    }

    fn install_cmake(&self) -> Result<(), failure::Error> {
        if is_bin_on_path("cmake")? {
            return Ok(());
        }

        println!("Installing cmake....");
        self.install_binary("cmake")
    }

    fn install_go(&self) -> Result<(), failure::Error> {
        if is_bin_on_path("go")? {
            return Ok(());
        }

        println!("Installing go....");
        match self {
            PackageManager::AptGet => Self::apt_get("golang"),
            PackageManager::Pacman => Self::pacman("go"),
            PackageManager::Yum => Self::yum("golang"),
            PackageManager::Brew => Self::brew("go"),
        }
    }

    fn install_unzip(&self) -> Result<(), failure::Error> {
        if is_bin_on_path("unzip")? {
            return Ok(());
        }

        println!("Installing unzip....");
        self.install_binary("unzip")
    }

    fn install_protobuf(&self) -> Result<(), failure::Error> {
        if is_bin_on_path("protoc")? {
            return Ok(());
        }

        println!("Installing protobuf....");
        match self {
            PackageManager::Brew => Self::brew("protobuf"),
            _ => {
                self.install_unzip()?;

                let protoc_version = "3.8.0";
                let protoc_zip = format!("protoc-{}-linux-x86_64.zip", protoc_version);
                let protoc_url = format!(
                    "https://github.com/google/protobuf/releases/download/v{}/{}",
                    protoc_version, protoc_zip
                );

                let status = Command::new("curl").arg("-OL").arg(protoc_url).status()?;
                if !status.success() {
                    bail!("Failed to install protoc");
                }

                let status = Command::new("sudo")
                    .arg("unzip")
                    .arg("-o")
                    .arg(&protoc_zip)
                    .arg("-d")
                    .arg("/usr/local")
                    .arg("bin/protoc")
                    .status()?;
                if !status.success() {
                    bail!("Failed to install protoc");
                }

                let status = Command::new("sudo")
                    .arg("unzip")
                    .arg("-o")
                    .arg(&protoc_zip)
                    .arg("-d")
                    .arg("/usr/local")
                    .arg("include/*")
                    .status()?;
                if !status.success() {
                    bail!("Failed to install protoc");
                }

                Command::new("rm").arg("-f").arg(&protoc_zip).status()?;
                if !status.success() {
                    bail!("Failed to install protoc");
                }

                println!("protoc is installed to /usr/local/bin/");

                Ok(())
            }
        }
    }
}

pub fn exec() -> Result<(), failure::Error> {
    println!(
        "\
Welcome to Libra!

This process will download and install the necessary dependencies needed to
build Libra Core. This includes:
    * cmake
    * protobuf
    * go

If you'd prefer to install these dependencies yourself, please exit now with
Ctrl-C."
    );

    print!("\nProceed with installing necessary dependencies? (y/N) > ");
    std::io::stdout().flush()?;
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    if !input.starts_with('y') {
        println!("Exiting...");
        return Ok(());
    }

    let package_manager = PackageManager::get_system_package_manager()?;
    package_manager.install_cmake()?;
    package_manager.install_go()?;
    package_manager.install_protobuf()?;

    println!(
        "
Finished installing all dependencies.

You should now be able to build the project by running:
    source {}/.cargo/env
    cargo build",
        std::env::var("HOME")?
    );

    Ok(())
}

fn is_bin_on_path<P: AsRef<Path>>(bin: P) -> Result<bool, failure::Error> {
    let output = Command::new("which").arg(bin.as_ref()).output()?;
    Ok(output.status.success())
}
