// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#[macro_use]
extern crate rental;

use guppy::graph::PackageGraph;
use once_cell::sync::OnceCell;
use std::path::{Path, PathBuf};

mod debug_ignore;
mod errors;
mod git;
mod graph;
mod workspace_subset;

use crate::git::GitCli;
pub use debug_ignore::*;
pub use errors::*;
use graph::PackageGraphPlus;
pub use workspace_subset::*;

/// Core context shared across all of x.
#[derive(Debug)]
pub struct XCoreContext {
    project_root: &'static Path,
    current_dir: PathBuf,
    current_rel_dir: PathBuf,
    git_cli: GitCli,
    package_graph_plus: DebugIgnore<OnceCell<PackageGraphPlus>>,
}

impl XCoreContext {
    /// Creates a new XCoreContext.
    pub fn new(project_root: &'static Path, current_dir: PathBuf) -> Result<Self> {
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
            current_dir,
            current_rel_dir,
            git_cli: GitCli::new(project_root)?,
            package_graph_plus: DebugIgnore(OnceCell::new()),
        })
    }

    /// Returns the project root for this workspace.
    pub fn project_root(&self) -> &'static Path {
        self.project_root
    }

    /// Returns the current working directory for this process.
    pub fn current_dir(&self) -> &Path {
        &self.current_dir
    }

    /// Returns the current working directory for this process, relative to the project root.
    pub fn current_rel_dir(&self) -> &Path {
        &self.current_rel_dir
    }

    /// Returns true if x has been run from the project root.
    pub fn current_dir_is_root(&self) -> bool {
        self.current_rel_dir == Path::new("")
    }

    /// Returns the Git CLI for this workspace.
    pub fn git_cli(&self) -> &GitCli {
        &self.git_cli
    }

    /// Returns the package graph for this workspace.
    pub fn package_graph(&self) -> Result<&PackageGraph> {
        Ok(self.package_graph_plus()?.head())
    }

    /// Returns information about the default members for this workspace.
    pub fn default_members<'a>(&'a self) -> Result<&'a WorkspaceSubset<'a>> {
        Ok(self.package_graph_plus()?.suffix())
    }

    // ---
    // Helper methods
    // ---

    fn package_graph_plus(&self) -> Result<&PackageGraphPlus> {
        self.package_graph_plus
            .get_or_try_init(|| PackageGraphPlus::create(self))
    }
}
