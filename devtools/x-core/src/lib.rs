// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{core_config::XCoreConfig, git::GitCli};
use camino::{Utf8Path, Utf8PathBuf};
use graph::PackageGraphPlus;
use guppy::graph::PackageGraph;
use hakari::{HakariBuilder, TomlOptions};
use once_cell::sync::OnceCell;

pub mod core_config;
mod debug_ignore;
mod errors;
pub mod git;
mod graph;
mod workspace_subset;

pub use debug_ignore::*;
pub use errors::*;
pub use workspace_subset::*;

/// Core context shared across all of x.
#[derive(Debug)]
pub struct XCoreContext {
    project_root: &'static Utf8Path,
    config: XCoreConfig,
    current_dir: Utf8PathBuf,
    current_rel_dir: Utf8PathBuf,
    git_cli: OnceCell<GitCli>,
    package_graph_plus: DebugIgnore<OnceCell<PackageGraphPlus>>,
}

impl XCoreContext {
    /// Creates a new XCoreContext.
    pub fn new(
        project_root: &'static Utf8Path,
        current_dir: Utf8PathBuf,
        config: XCoreConfig,
    ) -> Result<Self> {
        let current_rel_dir = match current_dir.strip_prefix(project_root) {
            Ok(rel_dir) => rel_dir.to_path_buf(),
            Err(_) => {
                return Err(SystemError::CwdNotInProjectRoot {
                    current_dir,
                    project_root,
                })
            }
        };
        // TODO: The project root should be managed by this struct, not by the global project_root
        // function.
        Ok(Self {
            project_root,
            config,
            current_dir,
            current_rel_dir,
            git_cli: OnceCell::new(),
            package_graph_plus: DebugIgnore(OnceCell::new()),
        })
    }

    /// Returns the project root for this workspace.
    pub fn project_root(&self) -> &'static Utf8Path {
        self.project_root
    }

    /// Returns the core config.
    pub fn config(&self) -> &XCoreConfig {
        &self.config
    }

    /// Returns the current working directory for this process.
    pub fn current_dir(&self) -> &Utf8Path {
        &self.current_dir
    }

    /// Returns the current working directory for this process, relative to the project root.
    pub fn current_rel_dir(&self) -> &Utf8Path {
        &self.current_rel_dir
    }

    /// Returns true if x has been run from the project root.
    pub fn current_dir_is_root(&self) -> bool {
        self.current_rel_dir == ""
    }

    /// Returns the Git CLI for this workspace.
    pub fn git_cli(&self) -> Result<&GitCli> {
        let root = self.project_root;
        self.git_cli.get_or_try_init(|| GitCli::new(root))
    }

    /// Returns the package graph for this workspace.
    pub fn package_graph(&self) -> Result<&PackageGraph> {
        Ok(self.package_graph_plus()?.package_graph())
    }

    /// For a given list of workspace packages, returns a tuple of (known, unknown) packages.
    ///
    /// Initializes the package graph if it isn't already done so, and returns an error if the
    pub fn partition_workspace_names<'a, B>(
        &self,
        names: impl IntoIterator<Item = &'a str>,
    ) -> Result<(B, B)>
    where
        B: Default + Extend<&'a str>,
    {
        let workspace = self.package_graph()?.workspace();
        let (known, unknown) = names
            .into_iter()
            .partition(|name| workspace.contains_name(name));
        Ok((known, unknown))
    }

    /// Returns information about the subsets for this workspace.
    pub fn subsets(&self) -> Result<&WorkspaceSubsets> {
        Ok(self.package_graph_plus()?.subsets())
    }

    /// Returns a Hakari builder for this workspace.
    pub fn hakari_builder(&self) -> Result<HakariBuilder<'_>> {
        let graph = self.package_graph()?;
        self.config
            .hakari
            .to_hakari_builder(graph)
            .map_err(|err| SystemError::guppy("while resolving Hakari config", err))
    }

    /// Returns the default Hakari TOML options for this workspace.
    pub fn hakari_toml_options(&self) -> TomlOptions {
        let mut options = TomlOptions::new();
        options.set_exact_versions(true);
        options
    }

    // ---
    // Helper methods
    // ---

    fn package_graph_plus(&self) -> Result<&PackageGraphPlus> {
        self.package_graph_plus
            .get_or_try_init(|| PackageGraphPlus::create(self))
    }
}
