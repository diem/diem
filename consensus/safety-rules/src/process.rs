// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    persistent_safety_storage::PersistentSafetyStorage,
    remote_service::{self, RemoteService},
    safety_rules_manager,
};
use consensus_types::common::{Author, Payload, Round};
use libra_config::config::{ConsensusType, NodeConfig, SafetyRulesService};
use libra_types::transaction::SignedTransaction;
use std::{marker::PhantomData, net::SocketAddr};

pub struct Process {
    consensus_type: ConsensusType,
    data: Option<ProcessData>,
}

impl Process {
    pub fn new(mut config: NodeConfig) -> Self {
        let (author, storage) = safety_rules_manager::extract_service_inputs(&mut config);

        let service = &config.consensus.safety_rules.service;
        let service = match &service {
            SafetyRulesService::Process(service) => service,
            SafetyRulesService::SpawnedProcess(service) => service,
            _ => panic!("Unexpected SafetyRules service: {:?}", service),
        };
        let server_addr = service.server_address;

        Self {
            consensus_type: service.consensus_type,
            data: Some(ProcessData {
                author,
                server_addr,
                storage,
            }),
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
        let data = self.data.take().expect("Unable to retrieve ProcessData");
        remote_service::execute::<T>(data.author, data.storage, data.server_addr);
    }
}

struct ProcessData {
    author: Author,
    server_addr: SocketAddr,
    storage: PersistentSafetyStorage,
}

pub struct ProcessService<T> {
    server_addr: SocketAddr,
    phantom_data: PhantomData<T>,
}

impl<T> ProcessService<T> {
    pub fn new(server_addr: SocketAddr) -> Self {
        Self {
            server_addr,
            phantom_data: PhantomData,
        }
    }
}

impl<T: Payload> RemoteService<T> for ProcessService<T> {
    fn server_address(&self) -> SocketAddr {
        self.server_addr
    }
}
