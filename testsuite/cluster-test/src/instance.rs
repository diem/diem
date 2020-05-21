// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use anyhow::{ensure, format_err, Result};
use libra_json_rpc_client::{JsonRpcAsyncClient, JsonRpcBatch};
use once_cell::sync::Lazy;
use regex::Regex;
use reqwest::Url;
use serde_json::Value;
use std::{collections::HashSet, ffi::OsStr, fmt, process::Stdio, str::FromStr};

static VAL_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"val-(\d+)").unwrap());
static FULLNODE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"fn-(\d+)").unwrap());

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum InstanceConfig {
    Validator(ValidatorConfig),
    Fullnode(FullnodeConfig),
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct ValidatorConfig {
    pub index: u32,
    pub num_validators: u32,
    pub num_fullnodes: u32,
    pub image_tag: String,
    pub config_overrides: Vec<String>,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct FullnodeConfig {
    pub fullnode_index: u32,
    pub num_fullnodes_per_validator: u32,
    pub validator_index: u32,
    pub num_validators: u32,
    pub image_tag: String,
    pub config_overrides: Vec<String>,
}

#[derive(Clone)]
pub struct Instance {
    peer_name: String,
    ip: String,
    ac_port: u32,
    k8s_node: Option<String>,
    instance_config: Option<InstanceConfig>,
}

impl Instance {
    pub fn new(peer_name: String, ip: String, ac_port: u32) -> Instance {
        Instance {
            peer_name,
            ip,
            ac_port,
            k8s_node: None,
            instance_config: None,
        }
    }

    pub fn new_k8s(
        peer_name: String,
        ip: String,
        ac_port: u32,
        k8s_node: Option<String>,
        instance_config: InstanceConfig,
    ) -> Instance {
        Instance {
            peer_name,
            ip,
            ac_port,
            k8s_node,
            instance_config: Some(instance_config),
        }
    }

    pub async fn run_cmd_tee_err<I, S>(&self, args: I) -> Result<()>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.run_cmd_inner(false, args).await
    }

    pub async fn run_cmd<I, S>(&self, args: I) -> Result<()>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.run_cmd_inner(true, args).await
    }

    pub async fn run_cmd_inner<I, S>(&self, no_std_err: bool, args: I) -> Result<()>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let ssh_dest = format!("ec2-user@{}", self.ip);
        let ssh_args = vec![
            "ssh",
            "-i",
            "/libra_rsa",
            "-oStrictHostKeyChecking=no",
            "-oConnectTimeout=3",
            "-oConnectionAttempts=10",
            ssh_dest.as_str(),
        ];
        let mut ssh_cmd = tokio::process::Command::new("timeout");
        ssh_cmd.arg("90").args(ssh_args).args(args);
        if no_std_err {
            ssh_cmd.stderr(Stdio::null());
        }
        let status = ssh_cmd.status().await?;
        ensure!(
            status.success(),
            "Failed with code {}",
            status.code().unwrap_or(-1)
        );
        Ok(())
    }

    pub async fn scp(&self, remote_file: &str, local_file: &str) -> Result<()> {
        let remote_file = format!("ec2-user@{}:{}", self.ip, remote_file);
        let status = tokio::process::Command::new("scp")
            .args(vec![
                "-i",
                "/libra_rsa",
                "-oStrictHostKeyChecking=no",
                "-oConnectTimeout=3",
                "-oConnectionAttempts=10",
                remote_file.as_str(),
                local_file,
            ])
            .status()
            .await?;
        ensure!(
            status.success(),
            "Failed with code {}",
            status.code().unwrap_or(-1)
        );
        Ok(())
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

    // TODO pass http client in here
    pub async fn try_json_rpc(&self) -> Result<()> {
        JsonRpcAsyncClient::new(self.json_rpc_url())
            .execute(JsonRpcBatch::new())
            .await?;
        Ok(())
    }

    pub fn peer_name(&self) -> &String {
        &self.peer_name
    }

    pub fn validator_index(&self) -> String {
        if let Some(cap) = VAL_REGEX.captures(&self.peer_name) {
            if let Some(cap) = cap.get(1) {
                return cap.as_str().to_string();
            }
        }
        if let Some(cap) = FULLNODE_REGEX.captures(&self.peer_name) {
            if let Some(cap) = cap.get(1) {
                return cap.as_str().to_string();
            }
        }
        panic!(
            "Failed to parse peer name {} into validator_index",
            self.peer_name
        )
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

    pub fn k8s_node(&self) -> Option<&String> {
        self.k8s_node.as_ref()
    }

    pub fn instance_config(&self) -> Option<&InstanceConfig> {
        self.instance_config.as_ref()
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

pub fn instance_configs(instances: &[Instance]) -> Result<Vec<&InstanceConfig>> {
    instances
        .iter()
        .map(|instance| -> Result<&InstanceConfig> {
            instance
                .instance_config()
                .ok_or_else(|| format_err!("Failed to find instance_config"))
        })
        .collect::<Result<_, _>>()
}
