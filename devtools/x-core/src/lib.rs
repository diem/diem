// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use guppy::{graph::PackageGraph, MetadataCommand};
use once_cell::sync::OnceCell;
use std::path::Path;

mod errors;

pub use errors::*;

/// Core context shared across all of x.
#[derive(Debug)]
pub struct XCoreContext {
    project_root: &'static Path,
    package_graph: OnceCell<PackageGraph>,
}

impl XCoreContext {
    /// Creates a new XCoreContext.
    pub fn new(project_root: &'static Path) -> Self {
        // TODO: The project root should be managed by this struct, not by the global project_root
        // function.
        Self {
            project_root,
            package_graph: OnceCell::new(),
        }
    }

    /// Returns the project root for this workspace.
    pub fn project_root(&self) -> &'static Path {
        self.project_root
    }

    /// Returns the package graph for this workspace.
    pub fn package_graph(&self) -> Result<&PackageGraph, guppy::Error> {
        self.package_graph.get_or_try_init(|| {
            let mut cmd = MetadataCommand::new();
            // Run cargo metadata from the root of the workspace.
            cmd.current_dir(self.project_root);
            Ok(cmd.build_graph()?)
        })
    }
}
