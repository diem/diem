// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0
use async_trait::async_trait;

pub mod k8s_node;
pub mod k8s_swarm;
pub mod process_node;
pub mod local_swarm;
pub mod instance_node;

use reqwest::Client;
use std::path::{PathBuf, Path};
use diem_config::config::RoleType;

#[derive(Debug, Clone)]
pub struct InstanceNodeParam {
    peer_name: String,
    ip: String,
    json_rpc_port: u32,
    debug_interface_port: Option<u32>,
    http_client: Client,
}

#[derive(Debug, Clone)]
pub struct ProcessNodeParam<'a> {
    diem_node_bin_path: &'a Path,
    node_id: String,
    role: RoleType,
    config_path: &'a Path,
    log_path: PathBuf,
}

#[derive(Debug, Clone)]
pub enum NodeParam<'a> {
    ProcessNode(ProcessNodeParam<'a>),
    InstanceNode(InstanceNodeParam),
}

#[async_trait]
pub trait Swarm {
}

#[async_trait]
pub trait Node {
    fn launch(&self, param: NodeParam) -> anyhow::Result<Box<Self>>;

    fn stop(&self) {}

    fn start(&self) {}

    fn json_rpc_port(&self) -> u32;

    fn debug_interface_port(&self) -> Option<u32>;
}
