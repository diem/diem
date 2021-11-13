// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use crate::cluster_swarm::cluster_swarm_kube::ClusterSwarmKube;
use anyhow::{format_err, Result};
use debug_interface::AsyncNodeDebugClient;
use diem_client::Client as JsonRpcClient;
use diem_config::config::NodeConfig;
use reqwest::{Client, Url};
use serde_json::Value;
use std::{
    collections::HashSet,
    fmt,
    process::Stdio,
    str::FromStr,
    time::{Duration, Instant},
};
use tokio::{process::Command, time};

#[derive(Debug, Clone, PartialEq)]
pub struct ValidatorGroup {
    pub index: u32,
    pub twin_index: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct InstanceConfig {
    pub validator_group: ValidatorGroup,
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
    pub image_tag: String,
    pub lsr_backend: String,
    pub vault_addr: Option<String>,
    pub vault_namespace: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ValidatorConfig {
    pub enable_lsr: bool,
    pub image_tag: String,
    pub safety_rules_addr: Option<String>,
    pub vault_addr: Option<String>,
    pub vault_namespace: Option<String>,
}

#[derive(Debug, Clone)]
pub struct FullnodeConfig {
    pub fullnode_index: u32,
    pub image_tag: String,
    pub seed_peer_ip: String,
    pub vault_addr: Option<String>,
    pub vault_namespace: Option<String>,
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

impl ValidatorGroup {
    pub fn new_for_index(index: u32) -> ValidatorGroup {
        Self {
            index,
            twin_index: None,
        }
    }

    pub fn index_only(&self) -> u32 {
        match self.twin_index {
            None => self.index,
            _ => panic!("Only validator has twin index"),
        }
    }
}

impl ApplicationConfig {
    pub fn needs_genesis(&self) -> bool {
        matches!(self, Self::Validator(_)) || matches!(self, Self::Fullnode(_))
    }

    pub fn needs_config(&self) -> bool {
        matches!(self, Self::Validator(_))
            || matches!(self, Self::Fullnode(_))
            || matches!(self, Self::LSR(_))
    }

    pub fn needs_fluentbit(&self) -> bool {
        matches!(self, Self::Validator(_))
            || matches!(self, Self::Fullnode(_))
            || matches!(self, Self::LSR(_))
    }
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
            ApplicationConfig::Validator(_) => match self.validator_group.twin_index {
                None => validator_pod_name(self.validator_group.index),
                twin_index => format!(
                    "val-{}-twin-{}",
                    self.validator_group.index,
                    twin_index.unwrap()
                ),
            },
            ApplicationConfig::Fullnode(fullnode_config) => {
                fullnode_pod_name(self.validator_group.index, fullnode_config.fullnode_index)
            }
            ApplicationConfig::LSR(_) => lsr_pod_name(self.validator_group.index),
            ApplicationConfig::Vault(_) => vault_pod_name(self.validator_group.index),
        }
    }

    pub fn make_twin(&mut self, twin_index: u32) {
        self.validator_group.twin_index = Some(twin_index);
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
            debug_interface_port,
            http_client,
            backend,
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
        self.json_rpc_client().batch(Vec::new()).await?;
        Ok(())
    }

    pub async fn wait_json_rpc(&self, deadline: Instant) -> Result<()> {
        loop {
            let ret = self.try_json_rpc().await;
            if ret.is_ok() {
                break;
            }
            if Instant::now() > deadline {
                return Err(format_err!(
                    "wait_json_rpc for {} timed out, last error: {:?}",
                    self,
                    ret.err().unwrap()
                ));
            }
            time::sleep(Duration::from_secs(3)).await;
        }
        Ok(())
    }

    pub fn peer_name(&self) -> &String {
        &self.peer_name
    }

    pub fn validator_group(&self) -> ValidatorGroup {
        self.k8s_backend().instance_config.validator_group.clone()
    }

    pub fn ip(&self) -> &String {
        &self.ip
    }

    pub fn ac_port(&self) -> u32 {
        self.ac_port
    }

    pub fn json_rpc_url(&self) -> Url {
        Url::from_str(&format!("http://{}:{}/v1", self.ip(), self.ac_port())).expect("Invalid URL.")
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

    pub fn json_rpc_client(&self) -> JsonRpcClient {
        JsonRpcClient::new(self.json_rpc_url().to_string())
    }

    pub async fn stop(&self) -> Result<()> {
        let backend = self.k8s_backend();
        backend.kube.delete_node(&backend.instance_config).await
    }

    /// Node must be stopped first
    pub async fn start(&self) -> Result<()> {
        let backend = self.k8s_backend();
        backend
            .kube
            .upsert_node(backend.instance_config.clone())
            .await
            .map(|_| ())
    }

    /// If deleting /opt/diem/data/* is required, call Instance::clean_date before calling
    /// Instance::start.
    pub async fn clean_data(&self) -> Result<()> {
        self.util_cmd("rm -rf /opt/diem/data/*; ", "clean-data")
            .await
    }

    pub async fn spawn_job(
        &self,
        docker_image: &str,
        command: &str,
        job_name: &str,
    ) -> Result<String> {
        let backend = self.k8s_backend();
        backend
            .kube
            .spawn_job(&backend.k8s_node, docker_image, command, job_name)
            .await
    }

    pub fn instance_config(&self) -> &InstanceConfig {
        let backend = self.k8s_backend();
        &backend.instance_config
    }

    pub async fn cmd<S: AsRef<str>>(
        &self,
        docker_image: &str,
        command: S,
        job_name: &str,
    ) -> Result<()> {
        let backend = self.k8s_backend();
        backend
            .kube
            .run(&backend.k8s_node, docker_image, command.as_ref(), job_name)
            .await
    }

    /// Runs command on the same host in separate utility container based on cluster-test-util image
    pub async fn util_cmd<S: AsRef<str>>(&self, command: S, job_name: &str) -> Result<()> {
        self.cmd(
            "853397791086.dkr.ecr.us-west-2.amazonaws.com/cluster-test-util:latest",
            command,
            job_name,
        )
        .await
    }

    /// Unlike util_cmd, exec runs command inside the container
    pub async fn exec(&self, command: &str, mute: bool) -> Result<()> {
        let mut cmd = Command::new("kubectl");
        cmd.arg("exec")
            .arg(&self.peer_name)
            .arg("--container")
            .arg("main")
            .arg("--")
            .arg("sh")
            .arg("-c")
            .arg(command)
            .kill_on_drop(true);
        if mute {
            cmd.stdout(Stdio::null()).stderr(Stdio::null());
        }
        let mut child = cmd.spawn().map_err(|e| {
            format_err!(
                "Failed to spawn child process {} on {}: {}",
                command,
                self.peer_name(),
                e
            )
        })?;
        let status = child
            .wait()
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
        write!(f, "{}", self)
    }
}

pub fn instancelist_to_set(instances: &[Instance]) -> HashSet<String> {
    let mut r = HashSet::new();
    for instance in instances {
        r.insert(instance.peer_name().clone());
    }
    r
}

pub fn validator_pod_name(index: u32) -> String {
    format!("val-{}", index)
}

pub fn vault_pod_name(index: u32) -> String {
    format!("vault-{}", index)
}

pub fn lsr_pod_name(index: u32) -> String {
    format!("lsr-{}", index)
}

pub fn fullnode_pod_name(validator_index: u32, fullnode_index: u32) -> String {
    format!("fn-{}-{}", validator_index, fullnode_index)
}
