// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::safety_rules_client::{
    local_client::LocalClient,
    remote_client::{LocalService, SerializerClient},
    spawned_process::SpawnedProcess,
};
use consensus_types::common::Author;
use libra_config::config::{NodeConfig, SafetyRulesService};
use safety_rules::{
    extract_service_inputs, PersistentSafetyStorage, ProcessService, SafetyRules,
    SerializerService, TSafetyRules, ThreadService,
};
use std::{
    net::SocketAddr,
    sync::{Arc, RwLock},
};

enum SafetyRulesWrapper {
    Local(Arc<RwLock<SafetyRules>>),
    Process(ProcessService),
    Serializer(Arc<RwLock<SerializerService>>),
    SpawnedProcess(SpawnedProcess),
    Thread(ThreadService),
}

pub struct SafetyRulesManager {
    internal_safety_rules: SafetyRulesWrapper,
}

impl SafetyRulesManager {
    pub fn new(config: &mut NodeConfig) -> Self {
        match &config.consensus.safety_rules.service {
            SafetyRulesService::Process(conf) => return Self::new_process(conf.server_address),
            SafetyRulesService::SpawnedProcess(_) => return Self::new_spawned_process(config),
            _ => (),
        };

        let (author, storage) = extract_service_inputs(config);
        let sr_config = &config.consensus.safety_rules;
        match sr_config.service {
            SafetyRulesService::Local => Self::new_local(author, storage),
            SafetyRulesService::Serializer => Self::new_serializer(author, storage),
            SafetyRulesService::Thread => Self::new_thread(author, storage),
            _ => panic!("Unimplemented SafetyRulesService: {:?}", sr_config.service),
        }
    }

    pub fn new_local(author: Author, storage: PersistentSafetyStorage) -> Self {
        let safety_rules = SafetyRules::new(author, storage);
        Self {
            internal_safety_rules: SafetyRulesWrapper::Local(Arc::new(RwLock::new(safety_rules))),
        }
    }

    pub fn new_process(server_addr: SocketAddr) -> Self {
        let process_service = ProcessService::new(server_addr);
        Self {
            internal_safety_rules: SafetyRulesWrapper::Process(process_service),
        }
    }

    pub fn new_serializer(author: Author, storage: PersistentSafetyStorage) -> Self {
        let safety_rules = SafetyRules::new(author, storage);
        let serializer_service = SerializerService::new(safety_rules);
        Self {
            internal_safety_rules: SafetyRulesWrapper::Serializer(Arc::new(RwLock::new(
                serializer_service,
            ))),
        }
    }

    pub fn new_spawned_process(config: &NodeConfig) -> Self {
        let process = SpawnedProcess::new(config);
        Self {
            internal_safety_rules: SafetyRulesWrapper::SpawnedProcess(process),
        }
    }

    pub fn new_thread(author: Author, storage: PersistentSafetyStorage) -> Self {
        let thread = ThreadService::new(author, storage);
        Self {
            internal_safety_rules: SafetyRulesWrapper::Thread(thread),
        }
    }

    pub fn client(&self) -> Box<dyn TSafetyRules + Send + Sync> {
        match &self.internal_safety_rules {
            SafetyRulesWrapper::Local(safety_rules) => {
                Box::new(LocalClient::new(safety_rules.clone()))
            }
            SafetyRulesWrapper::Process(process) => {
                Box::new(SerializerClient::new_from_remote_service(process))
            }
            SafetyRulesWrapper::Serializer(serializer_service) => {
                Box::new(SerializerClient::new_from_local_service(LocalService {
                    serializer_service: serializer_service.clone(),
                }))
            }
            SafetyRulesWrapper::SpawnedProcess(process) => {
                Box::new(SerializerClient::new_from_remote_service(process))
            }
            SafetyRulesWrapper::Thread(thread) => {
                Box::new(SerializerClient::new_from_remote_service(thread))
            }
        }
    }
}
