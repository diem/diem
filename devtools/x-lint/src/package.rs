// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{prelude::*, LintContext};
use guppy::{
    graph::{PackageGraph, PackageMetadata},
    PackageId,
};
use std::path::Path;

/// Represents a linter that runs once per package.
pub trait PackageLinter: Linter {
    fn run<'l>(
        &self,
        ctx: &PackageContext<'l>,
        out: &mut LintFormatter<'l, '_>,
    ) -> Result<RunStatus<'l>>;
}

/// Lint context for an individual package.
#[derive(Copy, Clone, Debug)]
pub struct PackageContext<'l> {
    project_ctx: ProjectContext<'l>,
    workspace_path: &'l Path,
    metadata: &'l PackageMetadata,
}

impl<'l> PackageContext<'l> {
    pub fn new(
        project_ctx: ProjectContext<'l>,
        package_graph: &'l PackageGraph,
        workspace_path: &'l Path,
        id: &PackageId,
    ) -> Self {
        Self {
            project_ctx,
            workspace_path,
            metadata: package_graph.metadata(id).expect("package id is valid"),
        }
    }

    /// Returns the project context
    pub fn project_ctx(&self) -> &ProjectContext<'l> {
        &self.project_ctx
    }

    /// Returns the relative path for this package in the workspace.
    pub fn workspace_path(&self) -> &'l Path {
        self.workspace_path
    }

    /// Returns the metadata for this package.
    pub fn metadata(&self) -> &'l PackageMetadata {
        self.metadata
    }
}

impl<'l> LintContext<'l> for PackageContext<'l> {
    fn kind(&self) -> LintKind<'l> {
        LintKind::Package {
            name: self.metadata.name(),
            workspace_path: self.workspace_path,
        }
    }
}
