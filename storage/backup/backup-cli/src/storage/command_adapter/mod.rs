// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

mod config;

#[cfg(test)]
mod tests;

use crate::storage::{
    command_adapter::config::{CommandAdapterConfig, EnvVar},
    BackupHandle, BackupHandleRef, BackupStorage, FileHandle, FileHandleRef, ShellSafeName,
};
use anyhow::{anyhow, ensure, Result};
use async_trait::async_trait;
use std::{path::PathBuf, process::Stdio};
use structopt::StructOpt;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite};

#[derive(StructOpt)]
pub struct CommandAdapterOpt {
    #[structopt(
        long = "config",
        help = "Config file for the command adapter backup store."
    )]
    config: PathBuf,
}

/// A BackupStorage that delegates required APIs to configured command lines.
/// see `CommandAdapterConfig`.
pub struct CommandAdapter {
    config: CommandAdapterConfig,
}

impl CommandAdapter {
    pub fn new(config: CommandAdapterConfig) -> Self {
        Self { config }
    }

    pub async fn new_with_opt(opt: CommandAdapterOpt) -> Result<Self> {
        let config = CommandAdapterConfig::load_from_file(&opt.config).await?;

        Ok(Self::new(config))
    }

    fn cmd(&self, cmd_str: &str, mut env_vars: Vec<EnvVar>) -> Command {
        env_vars.extend_from_slice(&self.config.env_vars);
        Command::new(cmd_str.to_string(), env_vars)
    }
}

#[async_trait]
impl BackupStorage for CommandAdapter {
    async fn create_backup(&self, name: &ShellSafeName) -> Result<BackupHandle> {
        let mut cmd = self.cmd(
            &self.config.commands.create_backup,
            vec![EnvVar::backup_name(name.to_string())],
        );
        let output = cmd.spawn().await?.wait_with_output().await?;
        ensure!(
            output.status.success(),
            "Failed running command: {:?}, Exit code: {:?}",
            cmd,
            output.status.code(),
        );
        let mut backup_handle = BackupHandle::from_utf8(output.stdout)?;
        backup_handle.truncate(backup_handle.trim_end().len());
        Ok(backup_handle)
    }

    async fn create_for_write(
        &self,
        backup_handle: &BackupHandleRef,
        name: &ShellSafeName,
    ) -> Result<(FileHandle, Box<dyn AsyncWrite + Send + Unpin>)> {
        let mut child = self
            .cmd(
                &self.config.commands.create_for_write,
                vec![
                    EnvVar::backup_handle(backup_handle.to_string()),
                    EnvVar::file_name(name.to_string()),
                ],
            )
            .spawn()
            .await?;
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| anyhow!("Child process stdin is None."))?;
        let mut stdout = child
            .stdout
            .take()
            .ok_or_else(|| anyhow!("Child process stdout is None."))?;
        let mut file_handle = FileHandle::new();
        stdout.read_to_string(&mut file_handle).await?;
        file_handle.truncate(file_handle.trim_end().len());
        Ok((file_handle, Box::new(stdin)))
    }

    async fn open_for_read(
        &self,
        file_handle: &FileHandleRef,
    ) -> Result<Box<dyn AsyncRead + Send + Unpin>> {
        let mut child = self
            .cmd(
                &self.config.commands.open_for_read,
                vec![EnvVar::file_handle(file_handle.to_string())],
            )
            .spawn()
            .await?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| anyhow!("Child process stdout is None."))?;
        Ok(Box::new(stdout))
    }
}

#[derive(Debug)]
struct Command {
    cmd_str: String,
    env_vars: Vec<EnvVar>,
}

impl Command {
    pub fn new(cmd_str: String, env_vars: Vec<EnvVar>) -> Self {
        Self { cmd_str, env_vars }
    }

    fn tokio_cmd(&self, cmd_str: &str) -> tokio::process::Command {
        let mut cmd = tokio::process::Command::new("sh");
        cmd.args(&["-c", cmd_str]);
        cmd.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit());

        for v in &self.env_vars {
            cmd.env(&v.key, &v.value);
        }

        cmd
    }

    async fn spawn(&mut self) -> Result<tokio::process::Child> {
        println!("Spawning {:?}", self);
        Ok(self.tokio_cmd(&self.cmd_str).spawn()?)
    }
}
