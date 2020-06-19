// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    persistent_safety_storage::PersistentSafetyStorage,
    remote_service::{self, RemoteService},
    safety_rules_manager,
};
use libra_config::config::{NodeConfig, SafetyRulesService};

use std::net::SocketAddr;

pub struct Process {
    data: Option<ProcessData>,
}

impl Process {
    pub fn new(mut config: NodeConfig) -> Self {
        let storage = safety_rules_manager::storage(&mut config);

        let service = &config.consensus.safety_rules.service;
        let service = match &service {
            SafetyRulesService::Process(service) => service,
            SafetyRulesService::SpawnedProcess(service) => service,
            _ => panic!("Unexpected SafetyRules service: {:?}", service),
        };
        let server_addr = service.server_address;

        Self {
            data: Some(ProcessData {
                server_addr,
                storage,
            }),
        }
    }

    pub fn start(&mut self) {
        let data = self.data.take().expect("Unable to retrieve ProcessData");
        remote_service::execute(data.storage, data.server_addr);
    }
}

struct ProcessData {
    server_addr: SocketAddr,
    storage: PersistentSafetyStorage,
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
