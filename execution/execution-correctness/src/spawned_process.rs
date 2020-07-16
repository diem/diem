// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::remote_service::RemoteService;
use libra_config::config::{ExecutionCorrectnessService, NodeConfig, PersistableConfig};
use libra_temppath::TempPath;
use std::{net::SocketAddr, process::Child};

pub struct SpawnedProcess {
    handle: Child,
    server_addr: SocketAddr,
    _config_path: TempPath,
    network_timeout: u64,
}

impl SpawnedProcess {
    pub fn new(config: &NodeConfig) -> Self {
        let mut config_path = TempPath::new();
        config_path.persist();
        config_path.create_as_file().unwrap();
        config.save_config(&config_path).unwrap();
        let service = &config.execution.service;
        let server_addr =
            if let ExecutionCorrectnessService::SpawnedProcess(remote_service) = service {
                remote_service.server_address
            } else {
                panic!("Invalid ExecutionCorrectnessService, expected SpawnedProcess.");
            };

        Self {
            handle: runner::run(&config_path.path()),
            server_addr,
            _config_path: config_path,
            network_timeout: config.execution.network_timeout_ms,
        }
    }
}

impl RemoteService for SpawnedProcess {
    fn server_address(&self) -> SocketAddr {
        self.server_addr
    }
    fn network_timeout(&self) -> u64 {
        self.network_timeout
    }
}

/// Kill ExecutionCorrectness process upon this object going out of scope
impl Drop for SpawnedProcess {
    fn drop(&mut self) {
        match self.handle.try_wait() {
            Ok(Some(_)) => {}
            _ => {
                if let Err(e) = self.handle.kill() {
                    panic!(
                        "Spawned ExecutionCorrectness process could not be killed: '{}'",
                        e
                    );
                }
            }
        }
    }
}

#[cfg(any(test, feature = "testing"))]
mod runner {
    const BINARY: &str = "execution-correctness";

    pub fn run(path: &std::path::Path) -> std::process::Child {
        let mut command = std::process::Command::new(workspace_builder::get_bin(BINARY));
        command
            .arg(path)
            .stdin(std::process::Stdio::inherit())
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit());
        command.spawn().unwrap()
    }
}

#[cfg(not(any(test, feature = "testing")))]
mod runner {
    pub fn run(_path: &std::path::Path) -> std::process::Child {
        panic!("Not supported outside of testing");
    }
}
