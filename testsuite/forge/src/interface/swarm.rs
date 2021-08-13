// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{ChainInfo, FullNode, NodeExt, Result, Validator, Version};
use anyhow::anyhow;
use diem_sdk::{client::BlockingClient, types::PeerId};
use std::{
    thread,
    time::{Duration, Instant},
};

/// Trait used to represent a running network comprised of Validators and FullNodes
pub trait Swarm {
    /// Performs a health check on the entire swarm, ensuring all Nodes are Live and that no forks
    /// have occurred
    fn health_check(&mut self) -> Result<()>;

    /// Returns an Iterator of references to all the Validators in the Swarm
    fn validators<'a>(&'a self) -> Box<dyn Iterator<Item = &'a dyn Validator> + 'a>;

    /// Returns an Iterator of mutable references to all the Validators in the Swarm
    fn validators_mut<'a>(&'a mut self) -> Box<dyn Iterator<Item = &'a mut dyn Validator> + 'a>;

    /// Returns a reference to the Validator with the provided PeerId
    fn validator(&self, id: PeerId) -> Option<&dyn Validator>;

    /// Returns a mutable reference to the Validator with the provided PeerId
    fn validator_mut(&mut self, id: PeerId) -> Option<&mut dyn Validator>;

    /// Upgrade a Validator to run specified `Version`
    fn upgrade_validator(&mut self, id: PeerId, version: &Version) -> Result<()>;

    /// Returns an Iterator of references to all the FullNodes in the Swarm
    fn full_nodes<'a>(&'a self) -> Box<dyn Iterator<Item = &'a dyn FullNode> + 'a>;

    /// Returns an Iterator of mutable references to all the FullNodes in the Swarm
    fn full_nodes_mut<'a>(&'a mut self) -> Box<dyn Iterator<Item = &'a mut dyn FullNode> + 'a>;

    /// Returns a reference to the FullNode with the provided PeerId
    fn full_node(&self, id: PeerId) -> Option<&dyn FullNode>;

    /// Returns a mutable reference to the FullNode with the provided PeerId
    fn full_node_mut(&mut self, id: PeerId) -> Option<&mut dyn FullNode>;

    /// Adds a Validator to the swarm with the provided PeerId
    fn add_validator(&mut self, id: PeerId) -> Result<PeerId>;

    /// Removes the Validator with the provided PeerId
    fn remove_validator(&mut self, id: PeerId) -> Result<()>;

    /// Adds a FullNode to the swarm with the provided PeerId
    fn add_full_node(&mut self, id: PeerId) -> Result<()>;

    /// Removes the FullNode with the provided PeerId
    fn remove_full_node(&mut self, id: PeerId) -> Result<()>;

    /// Return a list of supported Versions
    fn versions<'a>(&'a self) -> Box<dyn Iterator<Item = Version> + 'a>;

    /// Construct a ChainInfo from this Swarm
    fn chain_info(&mut self) -> ChainInfo<'_>;

    fn logs_location(&mut self) -> String;
}

impl<T: ?Sized> SwarmExt for T where T: Swarm {}

pub trait SwarmExt: Swarm {
    fn liveness_check(&self, deadline: Instant) -> Result<()> {
        let liveness_check_seconds = 10;
        let validators = self.validators().collect::<Vec<_>>();
        let full_nodes = self.full_nodes().collect::<Vec<_>>();

        while !validators
            .iter()
            .map(|node| node.liveness_check(liveness_check_seconds))
            .chain(
                full_nodes
                    .iter()
                    .map(|node| node.liveness_check(liveness_check_seconds)),
            )
            .all(|r| r.is_ok())
        {
            if Instant::now() > deadline {
                return Err(anyhow!("Swarm liveness check timed out"));
            }

            thread::sleep(Duration::from_millis(500));
        }

        Ok(())
    }

    /// Waits for the swarm to achieve connectivity
    fn wait_for_connectivity(&self, deadline: Instant) -> Result<()> {
        let validators = self.validators().collect::<Vec<_>>();
        let full_nodes = self.full_nodes().collect::<Vec<_>>();

        while !validators
            .iter()
            .map(|node| node.check_connectivity(validators.len() - 1))
            .chain(full_nodes.iter().map(|node| node.check_connectivity()))
            .all(|r| r.unwrap_or(false))
        {
            if Instant::now() > deadline {
                return Err(anyhow!("waiting for swarm connectivity timed out"));
            }

            thread::sleep(Duration::from_millis(500));
        }

        Ok(())
    }

    /// Perform a safety check, ensuring that no forks have occurred in the network.
    fn fork_check(&self) -> Result<()> {
        // Checks if root_hashes are equal across all nodes at a given version
        fn are_root_hashes_equal_at_version(
            clients: &[BlockingClient],
            version: u64,
        ) -> Result<bool> {
            let root_hashes = clients
                .iter()
                .map(|node| {
                    node.get_metadata_by_version(version)
                        .map(|r| r.into_inner().accumulator_root_hash)
                })
                .collect::<Result<Vec<_>, _>>()?;

            Ok(root_hashes.windows(2).all(|w| w[0] == w[1]))
        }

        let clients = self
            .validators()
            .map(|node| node.json_rpc_client())
            .chain(self.full_nodes().map(|node| node.json_rpc_client()))
            .collect::<Vec<_>>();

        let versions = clients
            .iter()
            .map(|node| node.get_metadata().map(|r| r.into_inner().version))
            .collect::<Result<Vec<_>, _>>()?;
        let min_version = versions
            .iter()
            .min()
            .copied()
            .ok_or_else(|| anyhow!("Unable to query nodes for their latest version"))?;
        let max_version = versions
            .iter()
            .max()
            .copied()
            .ok_or_else(|| anyhow!("Unable to query nodes for their latest version"))?;

        if !are_root_hashes_equal_at_version(&clients, min_version)? {
            return Err(anyhow!("Fork check failed"));
        }

        self.wait_for_all_nodes_to_catchup_to_version(
            max_version,
            Instant::now() + Duration::from_secs(10),
        )?;

        if !are_root_hashes_equal_at_version(&clients, max_version)? {
            return Err(anyhow!("Fork check failed"));
        }

        Ok(())
    }

    /// Waits for all nodes to have caught up to the specified `verison`.
    fn wait_for_all_nodes_to_catchup_to_version(
        &self,
        version: u64,
        deadline: Instant,
    ) -> Result<()> {
        let clients = self
            .validators()
            .map(|node| node.json_rpc_client())
            .chain(self.full_nodes().map(|node| node.json_rpc_client()))
            .collect::<Vec<_>>();

        while !clients
            .iter()
            .map(|node| node.get_metadata().map(|r| r.into_inner().version))
            .all(|maybe| maybe.map(|v| v as u64 >= version).unwrap_or(false))
        {
            if Instant::now() > deadline {
                return Err(anyhow!(
                    "waiting for nodes to catch up to version {} timed out",
                    version
                ));
            }

            thread::sleep(Duration::from_millis(500));
        }

        Ok(())
    }

    /// Wait for all nodes in the network to be caught up. This is done by first querying each node
    /// for its current version, selects the max version, then waits for all nodes to catch up to
    /// that version. Once done, we can guarantee that all transactions committed before invocation
    /// of this function are available at all the nodes in the swarm
    fn wait_for_all_nodes_to_catchup(&self, deadline: Instant) -> Result<()> {
        let clients = self
            .validators()
            .map(|node| node.json_rpc_client())
            .chain(self.full_nodes().map(|node| node.json_rpc_client()))
            .collect::<Vec<_>>();

        let latest_version = clients
            .iter()
            .map(|node| node.get_metadata().map(|r| r.into_inner().version))
            .map(|maybe| maybe.unwrap_or(0))
            .max()
            .ok_or_else(|| anyhow!("Unable to query nodes for their latest version"))?;

        self.wait_for_all_nodes_to_catchup_to_version(latest_version, deadline)
    }
}
