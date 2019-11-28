// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use failure::{self, prelude::*};
use std::collections::HashSet;
use std::{ffi::OsStr, fmt, process::Stdio};
use tokio;

#[derive(Clone)]
pub struct Instance {
    short_hash: String,
    ip: String,
    ac_port: u32,
}

impl Instance {
    pub fn new(short_hash: String, ip: String, ac_port: u32) -> Instance {
        Instance {
            short_hash,
            ip,
            ac_port,
        }
    }

    pub async fn run_cmd_tee_err<I, S>(&self, args: I) -> failure::Result<()>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.run_cmd_inner(false, args).await
    }

    pub async fn run_cmd<I, S>(&self, args: I) -> failure::Result<()>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.run_cmd_inner(true, args).await
    }

    pub async fn run_cmd_inner<I, S>(&self, no_std_err: bool, args: I) -> failure::Result<()>
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

    pub fn short_hash(&self) -> &String {
        &self.short_hash
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
        write!(f, "{}({})", self.short_hash, self.ip)
    }
}

pub fn instancelist_to_set(instances: &[Instance]) -> HashSet<String> {
    let mut r = HashSet::new();
    for instance in instances {
        r.insert(instance.short_hash().clone());
    }
    r
}
