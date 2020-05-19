// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{Result, SystemError};
use guppy::{
    graph::{
        cargo::{CargoOptions, CargoSet},
        feature::{default_filter, FeatureFilter, FeatureSet},
        PackageGraph, PackageMetadata,
    },
    PackageId,
};
use serde::Deserialize;
use std::{collections::HashSet, fs, path::Path};
use toml::de;

/// Information collected about a subset of members of a workspace.
///
/// Some subsets of this workspace have special properties that are enforced through linters.
pub struct WorkspaceSubset<'g> {
    package_graph: &'g PackageGraph,
    members: HashSet<&'g PackageId>,
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
        // For each workspace member, match the path to a package ID.
        let workspace = package_graph.workspace();
        let members = paths
            .into_iter()
            .map(|path| {
                Ok(workspace
                    .member_by_path(path)
                    .ok_or_else(|| {
                        // TODO: should have a custom error for this (move into guppy?)
                        guppy::Error::UnknownWorkspaceName(format!(
                            "unknown workspace member by path: {}",
                            path.display()
                        ))
                    })?
                    .id())
            })
            .collect::<Result<HashSet<_>, guppy::Error>>()?;
        let package_query = package_graph.query_forward(members.iter().copied())?;

        // Use the Cargo resolver to figure out which packages will be included.
        let build_set = package_graph
            .feature_graph()
            .query_packages(&package_query, feature_filter)
            .resolve_cargo(cargo_opts)?;
        let unified_set = build_set.host_features().union(build_set.target_features());

        Ok(Self {
            package_graph,
            members,
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

        let cargo_opts = CargoOptions::new().with_dev_deps(false);
        Ok(Self::new(
            package_graph,
            default_members,
            default_filter(),
            &cargo_opts,
        )?)
    }

    /// Returns true if the given package ID is a member of the build set, including transitive
    /// dependencies.
    pub fn contains(&self, package_id: &PackageId) -> bool {
        self.unified_set.features_for(package_id).is_some()
    }

    /// Returns true if the given package ID is a "root" member of this subset, ignoring transitive
    /// dependencies.
    pub fn is_root_member(&self, package_id: &PackageId) -> bool {
        self.members.contains(package_id)
    }

    /// Returns a list of root packages in this subset, ignoring transitive dependencies.
    pub fn root_members<'a>(&'a self) -> impl Iterator<Item = PackageMetadata<'g>> + 'a {
        let package_graph = self.package_graph;
        self.members.iter().map(move |package_id| {
            package_graph
                .metadata(package_id)
                .expect("valid package ID")
        })
    }

    /// Returns the set of packages and features that would be built from this subset.
    ///
    /// This contains information about transitive dependencies, both within the workspace and
    /// outside it.
    pub fn build_set(&self) -> &CargoSet<'g> {
        &self.build_set
    }

    /// Returns the set of packages and features that would be built from this subset, not making a
    /// distinction between host and target features.
    ///
    /// This contains information about transitive dependencies, both within the workspace and
    /// outside it.
    pub fn unified_set(&self) -> &FeatureSet<'g> {
        &self.unified_set
    }
}
