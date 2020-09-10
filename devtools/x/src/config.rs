// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{utils::project_root, Result};
use guppy::graph::summaries::CargoOptionsSummary;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
};

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
    /// Attributes to enforce on workspace crates
    pub enforced_attributes: EnforcedAttributesConfig,
    /// Banned dependencies
    pub banned_deps: BannedDepsConfig,
    /// Overlay config in this workspace
    pub overlay: OverlayConfig,
    /// Test-only config in this workspace
    pub test_only: TestOnlyConfig,
    /// Subsets of this workspace
    pub subsets: BTreeMap<String, SubsetConfig>,
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
pub struct OverlayConfig {
    /// A list of overlay feature names
    pub features: Vec<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct TestOnlyConfig {
    /// A list of test-only members
    pub members: HashSet<PathBuf>,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct SubsetConfig {
    /// The members in this subset
    pub members: HashSet<PathBuf>,
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
    pub toolchain: String,
    pub flags: Option<String>,
}

impl Config {
    pub fn from_file(f: impl AsRef<Path>) -> Result<Self> {
        let contents = fs::read(f)?;
        Self::from_toml(&contents).map_err(Into::into)
    }

    pub fn from_toml(bytes: &[u8]) -> Result<Self> {
        toml::from_slice(bytes).map_err(Into::into)
    }

    pub fn from_project_root() -> Result<Self> {
        Self::from_file(project_root().join("x.toml"))
    }

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
}
