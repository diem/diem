// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{Result, SystemError};
use guppy::{
    graph::{
        cargo::{CargoOptions, CargoResolverVersion, CargoSet},
        feature::{default_filter, FeatureFilter, FeatureSet},
        PackageGraph, PackageMetadata,
    },
    PackageId,
};
use serde::Deserialize;
use std::{fs, path::Path};
use toml::de;

/// Information collected about a subset of members of a workspace.
///
/// Some subsets of this workspace have special properties that are enforced through linters.
pub struct WorkspaceSubset<'g> {
    build_set: CargoSet<'g>,
    unified_set: FeatureSet<'g>,
}

impl<'g> WorkspaceSubset<'g> {
    /// Creates a new subset by simulating a Cargo build on the specified workspace paths, with
    /// the given feature filter.
    pub fn new<'a>(
        package_graph: &'g PackageGraph,
        paths: impl IntoIterator<Item = &'a Path>,
        feature_filter: impl FeatureFilter<'g>,
        cargo_opts: &CargoOptions<'_>,
    ) -> Result<Self, guppy::Error> {
        // Use the Cargo resolver to figure out which packages will be included.
        let build_set = package_graph
            .query_workspace_paths(paths)?
            .to_feature_query(feature_filter)
            .resolve_cargo(cargo_opts)?;
        let unified_set = build_set.host_features().union(build_set.target_features());

        Ok(Self {
            build_set,
            unified_set,
        })
    }

    /// Creates a new subset of default members by reading the root `Cargo.toml` file.
    ///
    /// This does not include dev-dependencies.
    pub fn default_members(package_graph: &'g PackageGraph, project_root: &Path) -> Result<Self> {
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

        let root_toml = project_root.join("Cargo.toml");
        let contents =
            fs::read(&root_toml).map_err(|err| SystemError::io("reading root Cargo.toml", err))?;
        let contents: RootToml = de::from_slice(&contents)
            .map_err(|err| SystemError::de("deserializing root Cargo.toml", err))?;
        let default_members = contents.workspace.default_members;

        let cargo_opts = CargoOptions::new()
            .with_version(CargoResolverVersion::V2)
            .with_dev_deps(false);
        Ok(Self::new(
            package_graph,
            default_members,
            default_filter(),
            &cargo_opts,
        )?)
    }

    /// Returns the status of the given package ID in the subset.
    pub fn status_of(&self, package_id: &PackageId) -> WorkspaceStatus {
        if self
            .build_set
            .original_query()
            .starts_from_package(package_id)
            .unwrap_or(false)
        {
            WorkspaceStatus::RootMember
        } else if self
            .unified_set
            .features_for(package_id)
            .unwrap_or(None)
            .is_some()
        {
            WorkspaceStatus::Dependency
        } else {
            WorkspaceStatus::Absent
        }
    }

    /// Returns a list of root packages in this subset, ignoring transitive dependencies.
    pub fn root_members<'a>(&'a self) -> impl Iterator<Item = PackageMetadata<'g>> + 'a {
        self.build_set.original_query().initial_packages()
    }

    /// Returns the set of packages and features that would be built from this subset.
    ///
    /// This contains information about transitive dependencies, both within the workspace and
    /// outside it.
    pub fn build_set(&self) -> &CargoSet<'g> {
        &self.build_set
    }
}

/// The status of a particular package ID in a `WorkspaceSubset`.
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub enum WorkspaceStatus {
    /// This package ID is a root member of the workspace subset.
    RootMember,
    /// This package ID is a dependency of the workspace subset, but not a root member.
    Dependency,
    /// This package ID is not a dependency of the workspace subset.
    Absent,
}
