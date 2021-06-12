// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::Result;
use debug_interface::NodeDebugClient;
use diem_client::Client as JsonRpcClient;
use diem_config::config::NodeConfig;
use diem_sdk::types::PeerId;
use reqwest::Url;

/// A NodeId is intended to be a unique identifier of a Node in a Swarm. Due to VFNs sharing the
/// same PeerId as their Validator, another identifier is needed in order to distinguish between
/// the two.
pub struct NodeId(usize);

impl NodeId {
    pub fn new(id: usize) -> Self {
        NodeId(id)
    }

    /// Accessor for the name of the module
    pub fn as_inner(&self) -> usize {
        self.0
    }
}

#[derive(Debug)]
pub enum HealthCheckError {
    NotRunning,
    RpcFailure(anyhow::Error),
    Unknown(anyhow::Error),
}

impl std::fmt::Display for HealthCheckError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for HealthCheckError {}

/// Trait used to represent a running Validator or FullNode
pub trait Node {
    /// Return the PeerId of this Node
    fn peer_id(&self) -> PeerId;

    /// Return the NodeId of this Node
    fn node_id(&self) -> NodeId;

    /// Return the URL for the JSON-RPC endpoint of this Node
    fn json_rpc_endpoint(&self) -> Url;

    /// Return JSON-RPC client of this Node
    fn json_rpc_client(&self) -> JsonRpcClient;

    /// Return a NodeDebugClient for this Node
    fn debug_client(&self) -> &NodeDebugClient;

    /// Query a Metric for from this Node
    fn get_metric(&mut self, metric_name: &str) -> Option<i64>;

    /// Return a reference to the Config this Node is using
    fn config(&self) -> &NodeConfig;

    /// Start this Node.
    /// This should be a noop if the Node is already running.
    fn start(&mut self) -> Result<()>;

    /// Stop this Node.
    /// This should be a noop if the Node isn't running.
    fn stop(&mut self) -> Result<()>;

    /// Restarts this Node by calling Node::Stop followed by Node::Start
    fn restart(&mut self) -> Result<()> {
        self.stop()?;
        self.start()
    }

    /// Clears this Node's Storage
    fn clear_storage(&mut self) -> Result<()>;

    /// Performs a Health Check on the Node
    fn health_check(&mut self) -> Result<(), HealthCheckError>;
}

/// Trait used to represent a running Validator
pub trait Validator: Node {}

/// Trait used to represent a running FullNode
pub trait FullNode: Node {}
