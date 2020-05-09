// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{prelude::*, LintContext};
use guppy::{
    graph::{DependencyDirection, PackageGraph},
    PackageId,
};
use once_cell::sync::OnceCell;
use serde::Deserialize;
use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};
use toml::de;
use x_core::XCoreContext;

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
    default_workspace_members: OnceCell<HashSet<PackageId>>,
}

impl<'l> ProjectContext<'l> {
    pub fn new(core: &'l XCoreContext) -> Self {
        Self {
            core,
            default_workspace_members: OnceCell::new(),
        }
    }

    /// Returns the project root.
    pub fn project_root(&self) -> &'l Path {
        self.core.project_root()
    }

    /// Returns the package graph, computing it for the first time if necessary.
    pub fn package_graph(&self) -> Result<&'l PackageGraph> {
        Ok(self.core.package_graph()?)
    }

    /// Returns the absolute path from the project root.
    pub fn full_path(&self, path: impl AsRef<Path>) -> PathBuf {
        self.core.project_root().join(path.as_ref())
    }

    /// Returns the default workspace members as a list of paths.
    ///
    /// TODO: This should be part of cargo metadata and made available through PackageGraph; see
    /// https://github.com/rust-lang/cargo/issues/8033.
    pub fn default_workspace_members(&self) -> Result<&HashSet<PackageId>> {
        self.default_workspace_members
            .get_or_try_init::<_, SystemError>(|| {
                #[derive(Deserialize)]
                struct RootToml<'a> {
                    #[serde(borrow)]
                    workspace: Workspace<'a>,
                }

                #[derive(Deserialize)]
                struct Workspace<'a> {
                    #[serde(borrow)]
                    #[serde(rename = "default-members")]
                    default_members: Vec<&'a Path>,
                }

                let root_toml = self.full_path("Cargo.toml");
                let contents = fs::read(&root_toml)
                    .map_err(|err| SystemError::io("reading root Cargo.toml", err))?;
                let contents: RootToml = de::from_slice(&contents)
                    .map_err(|err| SystemError::de("deserializing root Cargo.toml", err))?;
                let default_members = contents.workspace.default_members;

                // For each workspace member, match the path to a package ID.
                let package_graph = self.package_graph()?;
                let workspace = package_graph.workspace();
                let default_package_ids = default_members.iter().map(|path| {
                    workspace
                        .member_by_path(path)
                        .unwrap_or_else(|| {
                            panic!("unknown workspace member by path: {}", path.display())
                        })
                        .id()
                });
                let query = package_graph.query_forward(default_package_ids)?;
                let package_set = query.resolve_with_fn(|_, link| {
                    // Get all workspace IDs that are reachable from the default one. (Do not
                    // include dev-only dependencies).
                    !link.dev_only() && link.to().in_workspace()
                });
                Ok(package_set
                    .package_ids(DependencyDirection::Forward)
                    .cloned()
                    .collect())
            })
    }
}

impl<'l> LintContext<'l> for ProjectContext<'l> {
    fn kind(&self) -> LintKind<'l> {
        LintKind::Project
    }
}
