// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::storage::command_adapter::config::EnvVar;
use anyhow::Result;
use std::{
    fmt::{Debug, Formatter},
    process::Stdio,
};

pub(super) struct Command {
    cmd_str: String,
    env_vars: Vec<EnvVar>,
}

impl Command {
    pub fn new(raw_cmd: &str, env_vars: Vec<EnvVar>) -> Self {
        Self {
            cmd_str: format!("set -o nounset -o errexit -o pipefail; {}", raw_cmd),
            env_vars,
        }
    }

    fn tokio_cmd(&self) -> tokio::process::Command {
        let mut cmd = tokio::process::Command::new("bash");
        cmd.args(&["-c", &self.cmd_str]);
        cmd.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit());

        for v in &self.env_vars {
            cmd.env(&v.key, &v.value);
        }

        cmd
    }

    pub async fn spawn(&mut self) -> Result<tokio::process::Child> {
        println!("Spawning {:?}", self);
        Ok(self.tokio_cmd().spawn()?)
    }
}

impl Debug for Command {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            r#""{}" with env vars [{}]"#,
            self.cmd_str,
            self.env_vars
                .iter()
                .map(|v| format!("{}={}", v.key, v.value))
                .collect::<Vec<_>>()
                .join(", "),
        )
    }
}
