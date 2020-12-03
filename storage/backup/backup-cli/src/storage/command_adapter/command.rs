// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::storage::command_adapter::config::EnvVar;
use anyhow::{bail, ensure, Result};
use diem_logger::prelude::*;
use futures::{
    future::BoxFuture,
    task::{Context, Poll},
    Future, FutureExt,
};
use std::{
    fmt::{Debug, Formatter},
    process::Stdio,
};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    macros::support::Pin,
    process::{Child, ChildStdin, ChildStdout},
};

pub(super) struct Command {
    cmd_str: String,
    // API parameters, like $FILE_HANDLE for `open_for_read()`
    param_env_vars: Vec<EnvVar>,
    // Env vars defined in the config file.
    config_env_vars: Vec<EnvVar>,
}

impl Command {
    pub fn new(raw_cmd: &str, param_env_vars: Vec<EnvVar>, config_env_vars: Vec<EnvVar>) -> Self {
        Self {
            cmd_str: format!("set -o nounset -o errexit -o pipefail; {}", raw_cmd),
            param_env_vars,
            config_env_vars,
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
            r#""{}" with params [{}]"#,
            self.cmd_str,
            self.param_env_vars
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
        debug!("Spawning {:?}", command);

        let mut cmd = tokio::process::Command::new("bash");
        cmd.args(&["-c", &command.cmd_str]);
        cmd.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit());
        for v in command
            .config_env_vars
            .iter()
            .chain(command.param_env_vars.iter())
        {
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

    pub fn into_data_source<'a>(self) -> ChildStdoutAsDataSource<'a> {
        ChildStdoutAsDataSource::new(self)
    }

    pub fn into_data_sink<'a>(self) -> ChildStdinAsDataSink<'a> {
        ChildStdinAsDataSink::new(self)
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

pub(super) struct ChildStdoutAsDataSource<'a> {
    child: Option<SpawnedCommand>,
    join_fut: Option<BoxFuture<'a, Result<()>>>,
}

impl<'a> ChildStdoutAsDataSource<'a> {
    fn new(child: SpawnedCommand) -> Self {
        Self {
            child: Some(child),
            join_fut: None,
        }
    }
}

impl<'a> AsyncRead for ChildStdoutAsDataSource<'a> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<tokio::io::Result<usize>> {
        if self.child.is_some() {
            let res = Pin::new(self.child.as_mut().unwrap().stdout()).poll_read(cx, buf);
            if let Poll::Ready(Ok(0)) = res {
                // hit EOF, start joining self.child
                self.join_fut = Some(self.child.take().unwrap().join().boxed());
            } else {
                return res;
            }
        }

        Pin::new(self.join_fut.as_mut().unwrap())
            .poll(cx)
            .map_ok(|_| 0)
            .map_err(|e| tokio::io::Error::new(tokio::io::ErrorKind::Other, e))
    }
}

pub(super) struct ChildStdinAsDataSink<'a> {
    child: Option<SpawnedCommand>,
    join_fut: Option<BoxFuture<'a, Result<()>>>,
}

impl<'a> ChildStdinAsDataSink<'a> {
    fn new(child: SpawnedCommand) -> Self {
        Self {
            child: Some(child),
            join_fut: None,
        }
    }
}

impl<'a> AsyncWrite for ChildStdinAsDataSink<'a> {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, tokio::io::Error>> {
        if self.join_fut.is_some() {
            Poll::Ready(Err(tokio::io::ErrorKind::BrokenPipe.into()))
        } else {
            Pin::new(self.child.as_mut().unwrap().stdin()).poll_write(cx, buf)
        }
    }

    fn poll_flush(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), tokio::io::Error>> {
        if self.join_fut.is_some() {
            Poll::Ready(Err(tokio::io::ErrorKind::BrokenPipe.into()))
        } else {
            Pin::new(self.child.as_mut().unwrap().stdin()).poll_flush(cx)
        }
    }

    fn poll_shutdown(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), tokio::io::Error>> {
        if self.join_fut.is_none() {
            let res = Pin::new(self.child.as_mut().unwrap().stdin()).poll_shutdown(cx);
            if let Poll::Ready(Ok(_)) = res {
                // pipe shutdown successful
                self.join_fut = Some(self.child.take().unwrap().join().boxed())
            } else {
                return res;
            }
        }

        Pin::new(self.join_fut.as_mut().unwrap())
            .poll(cx)
            .map_err(|e| tokio::io::Error::new(tokio::io::ErrorKind::Other, e))
    }
}
