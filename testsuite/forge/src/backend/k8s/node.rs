// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{scale_sts_replica, FullNode, HealthCheckError, Node, Result, Validator, Version};
use anyhow::format_err;
use diem_config::config::NodeConfig;
use diem_sdk::{client::Client as JsonRpcClient, types::PeerId};
use reqwest::Url;
use std::{
    fmt::{Debug, Formatter},
    str::FromStr,
};
use tokio::runtime::Runtime;

pub struct K8sNode {
    pub(crate) name: String,
    pub(crate) sts_name: String,
    pub(crate) peer_id: PeerId,
    pub(crate) node_id: usize,
    pub(crate) dns: String,
    pub(crate) ip: String,
    pub(crate) port: u32,
    pub(crate) rest_api_port: u32,
    pub(crate) runtime: Runtime,
    pub version: Version,
}

impl K8sNode {
    fn port(&self) -> u32 {
        self.port
    }

    fn rest_api_port(&self) -> u32 {
        self.rest_api_port
    }

    #[allow(dead_code)]
    fn dns(&self) -> String {
        self.dns.clone()
    }

    fn ip(&self) -> String {
        self.ip.clone()
    }

    #[allow(dead_code)]
    fn node_id(&self) -> usize {
        self.node_id
    }

    pub(crate) fn json_rpc_client(&self) -> JsonRpcClient {
        JsonRpcClient::new(self.json_rpc_endpoint().to_string())
    }

    fn sts_name(&self) -> &str {
        &self.sts_name
    }
}

impl Node for K8sNode {
    fn peer_id(&self) -> PeerId {
        self.peer_id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn version(&self) -> Version {
        self.version.clone()
    }

    fn json_rpc_endpoint(&self) -> Url {
        Url::from_str(&format!("http://{}:{}/v1", self.ip(), self.port())).expect("Invalid URL.")
    }

    fn rest_api_endpoint(&self) -> Url {
        Url::from_str(&format!("http://{}:{}", self.ip(), self.rest_api_port()))
            .expect("Invalid URL.")
    }

    fn debug_endpoint(&self) -> Url {
        Url::parse(&format!("http://{}:{}", self.ip(), self.port())).unwrap()
    }

    fn config(&self) -> &NodeConfig {
        todo!()
    }

    fn scale_replicas(&self, replica_num: u64) -> Result<()> {
        scale_sts_replica(self.sts_name(), replica_num)
    }

    fn start(&mut self) -> Result<()> {
        println!("Going to start node {}", self.sts_name());
        self.scale_replicas(1)
    }

    fn stop(&mut self) -> Result<()> {
        println!("Going to stop node {}", self.sts_name());
        self.scale_replicas(0)
    }

    fn clear_storage(&mut self) -> Result<()> {
        todo!()
    }

    fn health_check(&mut self) -> Result<(), HealthCheckError> {
        let results = match self
            .runtime
            .block_on(self.json_rpc_client().batch(Vec::new()))
        {
            Ok(x) => x,
            Err(x) => return Err(HealthCheckError::RpcFailure(format_err!(x))),
        };
        if results.iter().all(Result::is_ok) {
            return Ok(());
        }
        Err(HealthCheckError::RpcFailure(format_err!(
            "K8s node health_check failed"
        )))
    }
}

impl Validator for K8sNode {}

impl FullNode for K8sNode {}

impl Debug for K8sNode {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}
