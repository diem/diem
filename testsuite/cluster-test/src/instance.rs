// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use crate::cluster_swarm::cluster_swarm_kube::ClusterSwarmKube;
use anyhow::{format_err, Result};
use debug_interface::AsyncNodeDebugClient;
use libra_config::config::NodeConfig;
use libra_json_rpc_client::{JsonRpcAsyncClient, JsonRpcBatch};
use reqwest::{Client, Url};
use serde_json::Value;
use std::{collections::HashSet, fmt, str::FromStr};
use tokio::process::Command;

#[derive(Debug, Clone)]
pub struct InstanceConfig {
    pub validator_group: u32,
    pub application_config: ApplicationConfig,
}

#[derive(Debug, Clone)]
pub enum ApplicationConfig {
    Validator(ValidatorConfig),
    Fullnode(FullnodeConfig),
    LSR(LSRConfig),
    Vault(VaultConfig),
}

#[derive(Debug, Clone)]
pub struct VaultConfig {}

#[derive(Debug, Clone)]
pub struct LSRConfig {
    pub num_validators: u32,
    pub image_tag: String,
    pub lsr_backend: String,
}

#[derive(Debug, Clone)]
pub struct ValidatorConfig {
    pub num_validators: u32,
    pub num_fullnodes: u32,
    pub enable_lsr: bool,
    pub image_tag: String,
    pub config_overrides: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct FullnodeConfig {
    pub fullnode_index: u32,
    pub num_fullnodes_per_validator: u32,
    pub num_validators: u32,
    pub image_tag: String,
    pub config_overrides: Vec<String>,
}

#[derive(Clone)]
pub struct Instance {
    peer_name: String,
    ip: String,
    ac_port: u32,
    debug_interface_port: Option<u32>,
    http_client: Client,
    backend: InstanceBackend,
}

#[derive(Clone)]
enum InstanceBackend {
    K8S(K8sInstanceInfo),
    Swarm,
}

#[derive(Clone)]
struct K8sInstanceInfo {
    k8s_node: String,
    instance_config: InstanceConfig,
    kube: ClusterSwarmKube,
}

impl InstanceConfig {
    pub fn replace_tag(&mut self, new_tag: String) -> Result<()> {
        match &mut self.application_config {
            ApplicationConfig::Validator(c) => {
                c.image_tag = new_tag;
            }
            ApplicationConfig::Fullnode(c) => {
                c.image_tag = new_tag;
            }
            ApplicationConfig::LSR(c) => {
                c.image_tag = new_tag;
            }
            ApplicationConfig::Vault(..) => {
                return Err(format_err!(
                    "InstanceConfig::Vault does not support custom tags"
                ));
            }
        }
        Ok(())
    }

    pub fn pod_name(&self) -> String {
        match &self.application_config {
            ApplicationConfig::Validator(_) => format!("val-{}", self.validator_group),
            ApplicationConfig::Fullnode(fullnode_config) => format!(
                "fn-{}-{}",
                self.validator_group, fullnode_config.fullnode_index
            ),
            ApplicationConfig::LSR(_) => format!("lsr-{}", self.validator_group),
            ApplicationConfig::Vault(_) => format!("vault-{}", self.validator_group),
        }
    }
}

impl Instance {
    pub fn new(
        peer_name: String,
        ip: String,
        ac_port: u32,
        debug_interface_port: Option<u32>,
        http_client: Client,
    ) -> Instance {
        let backend = InstanceBackend::Swarm;
        Instance {
            peer_name,
            ip,
            ac_port,
            backend,
            debug_interface_port,
            http_client,
        }
    }

    pub fn new_k8s(
        peer_name: String,
        ip: String,
        ac_port: u32,
        k8s_node: String,
        instance_config: InstanceConfig,
        http_client: Client,
        kube: ClusterSwarmKube,
    ) -> Instance {
        let backend = InstanceBackend::K8S(K8sInstanceInfo {
            k8s_node,
            instance_config,
            kube,
        });
        Instance {
            peer_name,
            ip,
            ac_port,
            debug_interface_port: Some(
                NodeConfig::default()
                    .debug_interface
                    .admission_control_node_debug_port as u32,
            ),
            http_client,
            backend,
        }
    }

    pub fn counter(&self, counter: &str) -> Result<f64> {
        let response: Value =
            reqwest::blocking::get(format!("http://{}:9101/counters", self.ip).as_str())?.json()?;
        if let Value::Number(ref response) = response[counter] {
            if let Some(response) = response.as_f64() {
                Ok(response)
            } else {
                Err(format_err!(
                    "Failed to parse counter({}) as f64: {:?}",
                    counter,
                    response
                ))
            }
        } else {
            Err(format_err!(
                "Counter({}) was not a Value::Number: {:?}",
                counter,
                response[counter]
            ))
        }
    }

    pub async fn try_json_rpc(&self) -> Result<()> {
        self.json_rpc_client().execute(JsonRpcBatch::new()).await?;
        Ok(())
    }

    pub fn peer_name(&self) -> &String {
        &self.peer_name
    }

    pub fn validator_group(&self) -> u32 {
        let backend = self.k8s_backend();
        backend.instance_config.validator_group
    }

    pub fn ip(&self) -> &String {
        &self.ip
    }

    pub fn ac_port(&self) -> u32 {
        self.ac_port
    }

    pub fn json_rpc_url(&self) -> Url {
        Url::from_str(&format!("http://{}:{}", self.ip(), self.ac_port())).expect("Invalid URL.")
    }

    fn k8s_backend(&self) -> &K8sInstanceInfo {
        if let InstanceBackend::K8S(ref k8s) = self.backend {
            return k8s;
        }
        panic!("Instance was not started with k8s");
    }

    pub fn debug_interface_port(&self) -> Option<u32> {
        self.debug_interface_port
    }

    pub fn json_rpc_client(&self) -> JsonRpcAsyncClient {
        JsonRpcAsyncClient::new_with_client(self.http_client.clone(), self.json_rpc_url())
    }

    pub async fn stop(&self) -> Result<()> {
        let backend = self.k8s_backend();
        backend.kube.delete_node(&backend.instance_config).await
    }

    /// Node must be stopped first
    pub async fn start(&self, delete_data: bool) -> Result<()> {
        let backend = self.k8s_backend();
        backend
            .kube
            .upsert_node(backend.instance_config.clone(), delete_data)
            .await
            .map(|_| ())
    }

    pub fn instance_config(&self) -> &InstanceConfig {
        let backend = self.k8s_backend();
        &backend.instance_config
    }

    /// Runs command on the same host in separate utility container based on cluster-test-util image
    pub async fn util_cmd<S: AsRef<str>>(&self, command: S, job_name: &str) -> Result<()> {
        let backend = self.k8s_backend();
        backend
            .kube
            .run(
                &backend.k8s_node,
                "853397791086.dkr.ecr.us-west-2.amazonaws.com/cluster-test-util:latest",
                command.as_ref(),
                job_name,
            )
            .await
    }

    /// Unlike util_cmd, exec runs command inside the container
    pub async fn exec(&self, command: &str) -> Result<()> {
        let child = Command::new("kubectl")
            .arg("exec")
            .arg(&self.peer_name)
            .arg("--container")
            .arg("main")
            .arg("--")
            .arg("sh")
            .arg("-c")
            .arg(command)
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| {
                format_err!(
                    "Failed to spawn child process {} on {}: {}",
                    command,
                    self.peer_name(),
                    e
                )
            })?;
        let status = child
            .await
            .map_err(|e| format_err!("Error running {} on {}: {}", command, self.peer_name(), e))?;
        if !status.success() {
            Err(format_err!(
                "Running {} on {}, exit code {:?}",
                command,
                self.peer_name(),
                status.code()
            ))
        } else {
            Ok(())
        }
    }

    pub fn debug_interface_client(&self) -> AsyncNodeDebugClient {
        AsyncNodeDebugClient::new(
            self.http_client.clone(),
            self.ip(),
            self.debug_interface_port
                .expect("debug_interface_port is not known on this instance") as u16,
        )
    }
}

impl fmt::Display for Instance {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}({})", self.peer_name, self.ip)
    }
}

impl fmt::Debug for Instance {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub fn instancelist_to_set(instances: &[Instance]) -> HashSet<String> {
    let mut r = HashSet::new();
    for instance in instances {
        r.insert(instance.peer_name().clone());
    }
    r
}
