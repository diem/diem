// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    execution_correctness_manager,
    remote_service::{self, RemoteService},
};
use diem_config::config::{ExecutionCorrectnessService, NodeConfig};
use diem_crypto::ed25519::Ed25519PrivateKey;
use std::net::SocketAddr;

pub struct Process {
    // TODO:  Restrict this to hold only the execution config.
    config: NodeConfig,
    prikey: Option<Ed25519PrivateKey>,
    // Timeout in milliseconds
    network_timeout_ms: u64,
}

impl Process {
    pub fn new(config: NodeConfig) -> Self {
        let prikey = execution_correctness_manager::extract_execution_prikey(&config);
        let network_timeout = config.execution.network_timeout_ms;
        Self {
            config,
            prikey,
            network_timeout_ms: network_timeout,
        }
    }

    pub fn start(self) {
        let service = &self.config.execution.service;
        let server_addr = match &service {
            ExecutionCorrectnessService::Process(remote_service) => remote_service.server_address,
            _ => panic!("Unexpected ExecutionCorrectness service: {:?}", service),
        };
        remote_service::execute(
            self.config.storage.address,
            server_addr,
            self.prikey,
            self.network_timeout_ms,
        );
    }
}

pub struct ProcessService {
    server_addr: SocketAddr,
    network_timeout: u64,
}

impl ProcessService {
    pub fn new(server_addr: SocketAddr, network_timeout: u64) -> Self {
        Self {
            server_addr,
            network_timeout,
        }
    }
}

impl RemoteService for ProcessService {
    fn server_address(&self) -> SocketAddr {
        self.server_addr
    }
    fn network_timeout(&self) -> u64 {
        self.network_timeout
    }
}
