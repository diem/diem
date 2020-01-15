// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use anyhow::{ensure, format_err, Result};
use serde_json::Value;
use std::collections::HashSet;
use std::{ffi::OsStr, fmt, process::Stdio};
use tokio;

#[derive(Clone)]
pub struct Instance {
    peer_name: String,
    ip: String,
    ac_port: u32,
}

impl Instance {
    pub fn new(peer_name: String, ip: String, ac_port: u32) -> Instance {
        Instance {
            peer_name,
            ip,
            ac_port,
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
        ssh_cmd.arg("60").args(ssh_args).args(args);
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

    pub fn is_up(&self) -> bool {
        reqwest::blocking::get(format!("http://{}:9101/counters", self.ip).as_str())
            .map(|x| x.status().is_success())
            .unwrap_or(false)
    }

    pub fn peer_name(&self) -> &String {
        &self.peer_name
    }

    pub fn ip(&self) -> &String {
        &self.ip
    }

    pub fn ac_port(&self) -> u32 {
        self.ac_port
    }
}

impl fmt::Display for Instance {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}({})", self.peer_name, self.ip)
    }
}

pub fn instancelist_to_set(instances: &[Instance]) -> HashSet<String> {
    let mut r = HashSet::new();
    for instance in instances {
        r.insert(instance.peer_name().clone());
    }
    r
}
