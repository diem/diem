// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{utils::project_root, Result};
use anyhow::Context;
use determinator::rules::DeterminatorRules;
use guppy::graph::summaries::CargoOptionsSummary;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
};
use x_core::core_config::XCoreConfig;

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct XConfig {
    /// Core configuration.
    #[serde(flatten)]
    pub core: XCoreConfig,
    /// X configuration.
    #[serde(flatten)]
    pub config: Config,
}

// TODO: probably split up lints and their configs into their own crate and section
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    /// Package exceptions which need to be run special
    system_tests: HashMap<String, Package>,
    /// Configuration for generating summaries
    summaries: SummariesConfig,
    /// Workspace configuration
    workspace: WorkspaceConfig,
    /// Clippy configureation
    clippy: Clippy,
    /// Fix configureation
    fix: Fix,
    /// Cargo configuration
    cargo: CargoConfig,
    grcov: CargoTool,
    /// Determinator configuration
    determinator: DeterminatorRules,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct CargoTool {
    pub installer: CargoInstallation,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct Sccache {
    /// Sccache Url
    pub installer: CargoInstallation,
    /// Where cargo home must reside for sccache to function properly.  Paths are embedded in binaries by rustc.
    pub required_cargo_home: String,
    /// Where the git repo for this project must reside.  Paths are embedded in binaries by rustc.
    pub required_git_home: String,
    /// s3 bucket location
    pub bucket: String,
    /// prefix to files uploaded in to s3
    pub prefix: Option<String>,
    /// utility of this seems to change in stable/vs rusoto, left for completeness.
    pub endpoint: Option<String>,
    /// AWS region of the bucket if necessary.
    pub region: Option<String>,
    /// Only used in stable, sscache delete when rusoto is merged in
    pub ssl: Option<bool>,
    /// If the bucket is public
    pub public: Option<bool>,
    /// Extra environment variables to set for the sccache server.
    pub envs: Option<Vec<(String, String)>>,
}
///
/// These can be passed to the installer.rs, which can check the installation against the version number supplied,
/// or install the cargo tool via either githash/repo if provided or with simply the version if the artifact is released
/// to crates.io.
///
/// Unfortunately there is no gaurantee that the installation is correct if the version numbers match as the githash
/// is not stored by default in the version number.
///
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct CargoInstallation {
    /// The version string that must match the installation, otherwise a fresh installation will occure.
    pub version: String,
    /// Overrides the default install with a specific git repo. git-rev is required.
    pub git: Option<String>,
    /// only used if the git url is set.  This is the full git hash.
    pub git_rev: Option<String>,
    /// features to enable in the installation.
    pub features: Option<Vec<String>>,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct Package {
    /// Path to the crate from root
    path: PathBuf,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct SummariesConfig {
    /// Config for default members and subsets
    pub default: CargoOptionsSummary,
    /// Config for the full workspace
    pub full: CargoOptionsSummary,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct WorkspaceConfig {
    /// Allowed characters in file paths. Regex must have ^ and $ anchors.
    pub allowed_paths: String,
    /// Attributes to enforce on workspace crates
    pub enforced_attributes: EnforcedAttributesConfig,
    /// Banned dependencies
    pub banned_deps: BannedDepsConfig,
    /// Direct dep duplicate lint config
    pub direct_dep_dups: DirectDepDupsConfig,
    /// Overlay config in this workspace
    pub overlay: OverlayConfig,
    /// Test-only config in this workspace
    pub test_only: TestOnlyConfig,
    /// Exceptions to whitespace linters
    pub whitespace_exceptions: Vec<String>,
    /// Move to Diem dependencies
    pub move_to_diem_deps: MoveToDiemDepsConfig,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct EnforcedAttributesConfig {
    /// Ensure the authors of every workspace crate are set to this.
    pub authors: Option<Vec<String>>,
    /// Ensure the `license` field of every workspace crate is set to this.
    pub license: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct BannedDepsConfig {
    /// Banned direct dependencies
    pub direct: HashMap<String, String>,
    /// Banned dependencies in the default build set
    pub default_build: HashMap<String, String>,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct DirectDepDupsConfig {
    pub allow: Vec<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct MoveToDiemDepsConfig {
    pub diem_crates_in_language: HashSet<String>,
    pub exclude: HashSet<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct OverlayConfig {
    /// A list of overlay feature names
    pub features: Vec<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct TestOnlyConfig {
    /// A list of test-only workspace names
    pub members: Vec<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct Clippy {
    allowed: Vec<String>,
    warn: Vec<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct Fix {}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct CargoConfig {
    pub sccache: Option<Sccache>,
}

impl XConfig {
    pub fn from_file(f: impl AsRef<Path>) -> Result<Self> {
        let f = f.as_ref();
        let contents =
            fs::read(f).with_context(|| format!("could not read config file {}", f.display()))?;
        Self::from_toml(&contents)
            .with_context(|| format!("could not parse config file {}", f.display()))
    }

    pub fn from_toml(bytes: &[u8]) -> Result<Self> {
        toml::from_slice(bytes).map_err(Into::into)
    }

    pub fn from_project_root() -> Result<Self> {
        Self::from_file(project_root().join("x.toml"))
    }
}

impl Config {
    pub fn cargo_config(&self) -> &CargoConfig {
        &self.cargo
    }

    pub fn system_tests(&self) -> &HashMap<String, Package> {
        &self.system_tests
    }

    pub fn summaries_config(&self) -> &SummariesConfig {
        &self.summaries
    }

    pub fn workspace_config(&self) -> &WorkspaceConfig {
        &self.workspace
    }

    pub fn allowed_clippy_lints(&self) -> &[String] {
        &self.clippy.allowed
    }

    pub fn warn_clippy_lints(&self) -> &[String] {
        &self.clippy.warn
    }

    pub fn tools(&self) -> Vec<(String, CargoInstallation)> {
        let mut tools = vec![("grcov".to_owned(), self.grcov().installer.to_owned())];
        if let Some(sccache) = &self.cargo_config().sccache {
            tools.push(("sccache".to_owned(), sccache.installer.to_owned()));
        }
        tools
    }

    pub fn grcov(&self) -> &CargoTool {
        &self.grcov
    }

    pub fn determinator_rules(&self) -> &DeterminatorRules {
        &self.determinator
    }
}
