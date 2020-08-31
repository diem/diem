// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::storage::command_adapter::config::EnvVar;
use anyhow::{bail, ensure, Result};
use std::{
    fmt::{Debug, Formatter},
    process::Stdio,
};
use tokio::process::{Child, ChildStdin, ChildStdout};

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

    pub fn spawn(self) -> Result<SpawnedCommand> {
        SpawnedCommand::spawn(self)
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

pub(super) struct SpawnedCommand {
    command: Command,
    child: Child,
}

impl SpawnedCommand {
    pub fn spawn(command: Command) -> Result<Self> {
        println!("Spawning {:?}", command);

        let mut cmd = tokio::process::Command::new("bash");
        cmd.args(&["-c", &command.cmd_str]);
        cmd.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit());
        for v in &command.env_vars {
            cmd.env(&v.key, &v.value);
        }
        let child = cmd.spawn()?;
        ensure!(child.stdin.is_some(), "child.stdin is None.");
        ensure!(child.stdout.is_some(), "child.stdout is None.");

        Ok(Self { command, child })
    }

    pub fn stdout(&mut self) -> &mut ChildStdout {
        self.child.stdout.as_mut().unwrap()
    }

    pub fn stdin(&mut self) -> &mut ChildStdin {
        self.child.stdin.as_mut().unwrap()
    }

    pub fn into_data_source(mut self) -> ChildStdout {
        self.child.stdout.take().unwrap()
    }

    pub fn into_data_sink(mut self) -> ChildStdin {
        self.child.stdin.take().unwrap()
    }

    pub async fn join(self) -> Result<()> {
        match self.child.wait_with_output().await {
            Ok(output) => {
                if output.status.success() {
                    Ok(())
                } else {
                    bail!(
                        "Command {:?} failed with exit status: {}",
                        self.command,
                        output.status
                    )
                }
            }
            Err(e) => bail!("Failed joining command {:?}: {}", self.command, e),
        }
    }
}
