// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{core_config::SubsetConfig, Result, SystemError};
use camino::{Utf8Path, Utf8PathBuf};
use guppy::{
    graph::{
        cargo::{CargoOptions, CargoResolverVersion, CargoSet},
        feature::{FeatureFilter, FeatureSet, StandardFeatures},
        DependencyDirection, PackageGraph, PackageMetadata, PackageSet,
    },
    PackageId,
};
use serde::Deserialize;
use std::{collections::BTreeMap, fs};
use toml::de;

/// Contains information about all the subsets specified in this workspace.
#[derive(Clone, Debug)]
pub struct WorkspaceSubsets<'g> {
    // TODO: default members should become a subset in x.toml
    default_members: WorkspaceSubset<'g>,
    subsets: BTreeMap<String, WorkspaceSubset<'g>>,
}

impl<'g> WorkspaceSubsets<'g> {
    /// Constructs a new store for workspace subsets.
    ///
    /// This is done with respect to a "standard build", which assumes:
    /// * any platform
    /// * v2 resolver
    /// * no dev dependencies
    pub fn new(
        graph: &'g PackageGraph,
        project_root: &Utf8Path,
        config: &BTreeMap<String, SubsetConfig>,
    ) -> Result<Self> {
        let mut cargo_opts = CargoOptions::new();
        cargo_opts
            .set_version(CargoResolverVersion::V2)
            .set_include_dev(false);

        let default_members = Self::read_default_members(project_root)?;

        // Look up default members by path.
        let initial_packages = graph
            .resolve_workspace_paths(&default_members)
            .map_err(|err| SystemError::guppy("querying default members", err))?;
        let default_members =
            WorkspaceSubset::new(&initial_packages, StandardFeatures::Default, &cargo_opts);

        // For each of the subset configs, look up the packages by name.
        let subsets = config
            .iter()
            .map(|(name, config)| {
                let initial_packages = graph
                    .resolve_workspace_names(&config.root_members)
                    .map_err(|err| {
                        SystemError::guppy(format!("querying members for subset '{}'", name), err)
                    })?;
                let subset =
                    WorkspaceSubset::new(&initial_packages, StandardFeatures::Default, &cargo_opts);
                Ok((name.clone(), subset))
            })
            .collect::<Result<_, _>>()?;

        Ok(Self {
            default_members,
            subsets,
        })
    }

    /// Returns information about default members.
    pub fn default_members(&self) -> &WorkspaceSubset<'g> {
        &self.default_members
    }

    /// Returns information about the subset by name.
    pub fn get(&self, name: impl AsRef<str>) -> Option<&WorkspaceSubset<'g>> {
        self.subsets.get(name.as_ref())
    }

    /// Iterate over all named subsets.
    pub fn iter<'a>(&'a self) -> impl Iterator<Item = (&'a str, &'a WorkspaceSubset<'g>)> + 'a {
        self.subsets
            .iter()
            .map(|(name, subset)| (name.as_str(), subset))
    }

    // ---
    // Helper methods
    // ---

    fn read_default_members(project_root: &Utf8Path) -> Result<Vec<Utf8PathBuf>> {
        #[derive(Deserialize)]
        struct RootToml {
            workspace: Workspace,
        }

        #[derive(Deserialize)]
        struct Workspace {
            #[serde(rename = "default-members")]
            default_members: Vec<Utf8PathBuf>,
        }

        let root_toml = project_root.join("Cargo.toml");
        let contents =
            fs::read(&root_toml).map_err(|err| SystemError::io("reading root Cargo.toml", err))?;
        let contents: RootToml = de::from_slice(&contents)
            .map_err(|err| SystemError::de("deserializing root Cargo.toml", err))?;
        Ok(contents.workspace.default_members)
    }
}

/// Information collected about a subset of members of a workspace.
///
/// Some subsets of this workspace have special properties that are enforced through linters.
#[derive(Clone, Debug)]
pub struct WorkspaceSubset<'g> {
    build_set: CargoSet<'g>,
    unified_set: FeatureSet<'g>,
}

impl<'g> WorkspaceSubset<'g> {
    /// Creates a new subset by simulating a Cargo build on the specified workspace paths, with
    /// the given feature filter.
    pub fn new<'a>(
        initial_packages: &PackageSet<'g>,
        feature_filter: impl FeatureFilter<'g>,
        cargo_opts: &CargoOptions<'_>,
    ) -> Self {
        // Use the Cargo resolver to figure out which packages will be included.
        let build_set = initial_packages
            .to_feature_set(feature_filter)
            .into_cargo_set(cargo_opts)
            .expect("into_cargo_set should always succeed");
        let unified_set = build_set.host_features().union(build_set.target_features());

        Self {
            build_set,
            unified_set,
        }
    }

    /// Returns the initial members that this subset was constructed from.
    pub fn initials(&self) -> &FeatureSet<'g> {
        self.build_set.initials()
    }

    /// Returns the status of the given package ID in the subset.
    pub fn status_of(&self, package_id: &PackageId) -> WorkspaceStatus {
        if self
            .build_set
            .initials()
            .contains_package(package_id)
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
        self.build_set
            .initials()
            .packages_with_features(DependencyDirection::Forward)
            .map(|f| *f.package())
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
