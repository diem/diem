// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    execution_correctness_manager,
    remote_service::{self, RemoteService},
};
use libra_config::config::{ExecutionCorrectnessService, NodeConfig};
use libra_crypto::ed25519::Ed25519PrivateKey;
use std::net::SocketAddr;

pub struct Process {
    config: NodeConfig,
    prikey: Option<Ed25519PrivateKey>,
}

impl Process {
    pub fn new(mut config: NodeConfig) -> Self {
        let prikey = execution_correctness_manager::extract_execution_prikey(&mut config);
        Self { config, prikey }
    }

    pub fn start(self) {
        let service = &self.config.execution.service;
        let server_addr = match &service {
            ExecutionCorrectnessService::Process(remote_service) => remote_service.server_address,
            ExecutionCorrectnessService::SpawnedProcess(remote_service) => {
                remote_service.server_address
            }
            _ => panic!("Unexpected ExecutionCorrectness service: {:?}", service),
        };
        remote_service::execute(self.config.storage.address, server_addr, self.prikey);
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
