// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    persistent_storage::PersistentStorage,
    remote_service::{self, RemoteService},
    safety_rules_manager,
};
use consensus_types::common::{Author, Payload, Round};
use libra_config::config::{ConsensusType, NodeConfig, SafetyRulesService};
use libra_types::transaction::SignedTransaction;
use std::net::SocketAddr;

pub struct ProcessService {
    consensus_type: ConsensusType,
    data: Option<ProcessServiceData>,
    server_addr: SocketAddr,
}

impl ProcessService {
    pub fn new(mut config: NodeConfig) -> Self {
        let (author, storage) = safety_rules_manager::extract_service_inputs(&mut config);

        let service = &config.consensus.safety_rules.service;
        let service = match &service {
            SafetyRulesService::Process(service) => service,
            SafetyRulesService::SpawnedProcess(service) => service,
            _ => panic!("Unexpected SafetyRules service: {:?}", service),
        };

        Self {
            consensus_type: service.consensus_type,
            data: Some(ProcessServiceData { author, storage }),
            server_addr: service.server_address,
        }
    }

    pub fn start(&mut self) {
        match self.consensus_type {
            ConsensusType::Bytes => self.start_internal::<Vec<u8>>(),
            ConsensusType::Rounds => self.start_internal::<Round>(),
            ConsensusType::SignedTransactions => self.start_internal::<Vec<SignedTransaction>>(),
        }
    }

    fn start_internal<T: Payload>(&mut self) {
        let data = self
            .data
            .take()
            .expect("Unable to retrieve ProcessServiceData");
        remote_service::execute::<T>(data.author, data.storage, self.server_addr);
    }
}

impl<T: Payload> RemoteService<T> for ProcessService {
    fn server_address(&self) -> SocketAddr {
        self.server_addr
    }
}

struct ProcessServiceData {
    author: Author,
    storage: PersistentStorage,
}
