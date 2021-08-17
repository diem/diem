// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{Factory, Result, Swarm, Version};
use anyhow::Context;
use rand::rngs::StdRng;
use std::{
    collections::HashMap,
    num::NonZeroUsize,
    path::{Path, PathBuf},
    sync::Arc,
};

mod cargo;
mod node;
mod swarm;
pub use node::LocalNode;
pub use swarm::{LocalSwarm, LocalSwarmBuilder, SwarmDirectory};

#[derive(Clone, Debug)]
pub struct LocalVersion {
    revision: String,
    bin: PathBuf,
    version: Version,
}

impl LocalVersion {
    fn bin(&self) -> &Path {
        &self.bin
    }

    fn version(&self) -> Version {
        self.version.clone()
    }
}

pub struct LocalFactory {
    versions: Arc<HashMap<Version, LocalVersion>>,
}

impl LocalFactory {
    pub fn new(versions: HashMap<Version, LocalVersion>) -> Self {
        Self {
            versions: Arc::new(versions),
        }
    }

    pub fn from_workspace() -> Result<Self> {
        let mut versions = HashMap::new();
        let new_version = cargo::get_diem_node_binary_from_worktree().map(|(revision, bin)| {
            let version = Version::new(usize::max_value(), revision.clone());
            LocalVersion {
                revision,
                bin,
                version,
            }
        })?;

        versions.insert(new_version.version.clone(), new_version);
        Ok(Self::new(versions))
    }

    pub fn from_revision(revision: &str) -> Result<Self> {
        let mut versions = HashMap::new();
        let new_version =
            cargo::get_diem_node_binary_at_revision(revision).map(|(revision, bin)| {
                let version = Version::new(usize::max_value(), revision.clone());
                LocalVersion {
                    revision,
                    bin,
                    version,
                }
            })?;

        versions.insert(new_version.version.clone(), new_version);
        Ok(Self::new(versions))
    }

    pub fn with_revision_and_workspace(revision: &str) -> Result<Self> {
        let workspace = cargo::get_diem_node_binary_from_worktree().map(|(revision, bin)| {
            let version = Version::new(usize::max_value(), revision.clone());
            LocalVersion {
                revision,
                bin,
                version,
            }
        })?;
        let revision =
            cargo::get_diem_node_binary_at_revision(revision).map(|(revision, bin)| {
                let version = Version::new(usize::min_value(), revision.clone());
                LocalVersion {
                    revision,
                    bin,
                    version,
                }
            })?;

        let mut versions = HashMap::new();
        versions.insert(workspace.version(), workspace);
        versions.insert(revision.version(), revision);
        Ok(Self::new(versions))
    }

    /// Create a LocalFactory with a diem-node version built at the tip of upstream/main and the
    /// current workspace, suitable for compatibility testing.
    pub fn with_upstream_and_workspace() -> Result<Self> {
        let upstream_main = cargo::git_get_upstream_remote().map(|r| format!("{}/main", r))?;
        Self::with_revision_and_workspace(&upstream_main)
    }

    /// Create a LocalFactory with a diem-node version built at merge-base of upstream/main and the
    /// current workspace, suitable for compatibility testing.
    pub fn with_upstream_merge_base_and_workspace() -> Result<Self> {
        let upstream_main = cargo::git_get_upstream_remote().map(|r| format!("{}/main", r))?;
        let merge_base = cargo::git_merge_base(upstream_main)?;
        Self::with_revision_and_workspace(&merge_base)
    }
}

impl Factory for LocalFactory {
    fn versions<'a>(&'a self) -> Box<dyn Iterator<Item = Version> + 'a> {
        Box::new(self.versions.keys().cloned())
    }

    fn launch_swarm(
        &self,
        rng: &mut StdRng,
        node_num: NonZeroUsize,
        version: &Version,
    ) -> Result<Box<dyn Swarm>> {
        let mut swarm = LocalSwarm::builder(self.versions.clone())
            .number_of_validators(node_num)
            .initial_version(version.clone())
            .build(rng)?;
        swarm
            .launch()
            .with_context(|| format!("Swarm logs can be found here: {}", swarm.logs_location()))?;

        // swarm.launch().map_err(|e| {
        //     swarm.logs_location()
        //     e
        // });
        // match swarm.launch() {
        //     Ok(()) => {}
        //     Err(e) => {
        //         swarm.
        //         return e;
        //     }
        // }

        Ok(Box::new(swarm))
    }
}
