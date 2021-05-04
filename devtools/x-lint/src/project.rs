// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{prelude::*, LintContext};
use camino::{Utf8Path, Utf8PathBuf};
use guppy::graph::PackageGraph;
use hakari::Hakari;
use x_core::{WorkspaceSubset, XCoreContext};

/// Represents a linter that checks some property for the overall project.
///
/// Linters that implement `ProjectLinter` will run once for the whole project.
pub trait ProjectLinter: Linter {
    // Since ProjectContext is only 1 word long, clippy complains about passing it by reference. Do
    // it that way for consistency reasons.
    #[allow(clippy::trivially_copy_pass_by_ref)]
    /// Executes the lint against the given project context.
    fn run<'l>(
        &self,
        ctx: &ProjectContext<'l>,
        out: &mut LintFormatter<'l, '_>,
    ) -> Result<RunStatus<'l>>;
}

/// Overall linter context for a project.
#[derive(Debug)]
pub struct ProjectContext<'l> {
    core: &'l XCoreContext,
}

impl<'l> ProjectContext<'l> {
    pub fn new(core: &'l XCoreContext) -> Self {
        Self { core }
    }

    /// Returns the core context.
    pub fn core(&self) -> &'l XCoreContext {
        self.core
    }

    /// Returns the project root.
    pub fn project_root(&self) -> &'l Utf8Path {
        self.core.project_root()
    }

    /// Returns the package graph, computing it for the first time if necessary.
    pub fn package_graph(&self) -> Result<&'l PackageGraph> {
        self.core.package_graph()
    }

    /// Returns the absolute path from the project root.
    pub fn full_path(&self, path: impl AsRef<Utf8Path>) -> Utf8PathBuf {
        self.core.project_root().join(path.as_ref())
    }

    /// Returns information about the default workspace members.
    ///
    /// This includes all packages included by default in the default workspace members, but not
    /// those that Cargo would ignore.
    pub fn default_members(&self) -> Result<&WorkspaceSubset> {
        Ok(self.core.subsets()?.default_members())
    }

    /// Returns Hakari information.
    pub fn hakari(&self) -> Result<Hakari<'l, 'static>> {
        Ok(self.core.hakari_builder()?.compute())
    }
}

impl<'l> LintContext<'l> for ProjectContext<'l> {
    fn kind(&self) -> LintKind<'l> {
        LintKind::Project
    }
}
