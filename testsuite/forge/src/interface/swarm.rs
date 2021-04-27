// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{AdminInfo, FullNode, NodeId, PublicInfo, Result, Validator};

/// Trait used to represent a running network comprised of Validators and FullNodes
pub trait Swarm {
    /// Performs a health check on the entire swarm, ensuring all Nodes are Live and that no forks
    /// have occurred
    fn health_check(&mut self) -> Result<()>;

    /// Returns an Iterator of references to all the Validators in the Swarm
    fn validators<'a>(&'a mut self) -> Box<dyn Iterator<Item = &'a dyn Validator> + 'a>;

    /// Returns an Iterator of mutable references to all the Validators in the Swarm
    fn validators_mut<'a>(&'a mut self) -> Box<dyn Iterator<Item = &'a mut dyn Validator> + 'a>;

    /// Returns a reference to the Validator with the provided NodeId
    fn validator(&self, id: NodeId) -> &dyn Validator;

    /// Returns a mutable reference to the Validator with the provided NodeId
    fn validator_mut(&mut self, id: NodeId) -> &mut dyn Validator;

    /// Returns an Iterator of references to all the FullNodes in the Swarm
    fn full_nodes<'a>(&'a mut self) -> Box<dyn Iterator<Item = &'a dyn FullNode> + 'a>;

    /// Returns an Iterator of mutable references to all the FullNodes in the Swarm
    fn full_nodes_mut<'a>(&'a mut self) -> Box<dyn Iterator<Item = &'a mut dyn FullNode> + 'a>;

    /// Returns a reference to the FullNode with the provided NodeId
    fn full_node(&self, id: NodeId) -> &dyn FullNode;

    /// Returns a mutable reference to the FullNode with the provided NodeId
    fn full_node_mut(&mut self, id: NodeId) -> &mut dyn FullNode;

    /// Adds a Validator to the swarm with the provided NodeId
    fn add_validator(&mut self, id: NodeId) -> Result<NodeId>;

    /// Removes the Validator with the provided NodeId
    fn remove_validator(&mut self, id: NodeId) -> Result<()>;

    /// Adds a FullNode to the swarm with the provided NodeId
    fn add_full_node(&mut self, id: NodeId) -> Result<()>;

    /// Removes the FullNode with the provided NodeId
    fn remove_full_node(&mut self, id: NodeId) -> Result<()>;

    /// Construct an AdminInfo from this Swarm
    fn admin_info(&mut self) -> AdminInfo<'_>;

    /// Construct a PublicInfo from this Swarm
    fn public_info(&mut self) -> PublicInfo<'_>;
}
