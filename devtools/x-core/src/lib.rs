// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#[macro_use]
extern crate rental;

use guppy::graph::PackageGraph;
use once_cell::sync::OnceCell;
use std::path::Path;

mod debug_ignore;
mod errors;
mod graph;
mod workspace_subset;

pub use debug_ignore::*;
pub use errors::*;
use graph::PackageGraphPlus;
pub use workspace_subset::*;

/// Core context shared across all of x.
#[derive(Debug)]
pub struct XCoreContext {
    project_root: &'static Path,
    package_graph_plus: DebugIgnore<OnceCell<PackageGraphPlus>>,
}

impl XCoreContext {
    /// Creates a new XCoreContext.
    pub fn new(project_root: &'static Path) -> Self {
        // TODO: The project root should be managed by this struct, not by the global project_root
        // function.
        Self {
            project_root,
            package_graph_plus: DebugIgnore(OnceCell::new()),
        }
    }

    /// Returns the project root for this workspace.
    pub fn project_root(&self) -> &'static Path {
        self.project_root
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
