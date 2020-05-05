// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::remote_service::{self, RemoteService};
use libra_config::config::{ExecutionCorrectnessService, NodeConfig};
use std::net::SocketAddr;

pub struct Process {
    config: NodeConfig,
}

impl Process {
    pub fn new(config: NodeConfig) -> Self {
        Self { config }
    }

    pub fn start(&self) {
        let service = &self.config.execution.service;
        let server_addr = match &service {
            ExecutionCorrectnessService::Process(remote_service) => remote_service.server_address,
            ExecutionCorrectnessService::SpawnedProcess(remote_service) => {
                remote_service.server_address
            }
            _ => panic!("Unexpected ExecutionCorrectness service: {:?}", service),
        };
        remote_service::execute(self.config.storage.simple_address, server_addr);
    }
}

pub struct ProcessService {
    server_addr: SocketAddr,
}

impl ProcessService {
    pub fn new(server_addr: SocketAddr) -> Self {
        Self { server_addr }
    }
}

impl RemoteService for ProcessService {
    fn server_address(&self) -> SocketAddr {
        self.server_addr
    }
}
