// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    local_client::LocalClient,
    persistent_safety_storage::PersistentSafetyStorage,
    process::ProcessService,
    remote_service::RemoteService,
    serializer::{SerializerClient, SerializerService},
    spawned_process::SpawnedProcess,
    thread::ThreadService,
    SafetyRules, TSafetyRules,
};
use libra_config::config::{NodeConfig, SafetyRulesService};
use libra_secure_storage::{KVStorage, Storage};
use std::{
    convert::TryInto,
    net::SocketAddr,
    sync::{Arc, RwLock},
};

pub fn storage(config: &mut NodeConfig) -> PersistentSafetyStorage {
    let backend = &config.consensus.safety_rules.backend;
    let internal_storage: Storage = backend.try_into().expect("Unable to initialize storage");
    internal_storage
        .available()
        .expect("Storage is not available");

    if let Some(test_config) = config.test.as_mut() {
        let author = config
            .validator_network
            .as_ref()
            .expect("Missing validator network")
            .peer_id();

        let consensus_private_key = test_config
            .consensus_keypair
            .as_mut()
            .expect("Missing consensus keypair in test config")
            .take_private()
            .expect("Failed to take Consensus private key, key absent or already read");
        let waypoint = config.base.waypoint.waypoint();

        let execution_public_key = test_config
            .execution_keypair
            .as_mut()
            .expect("Missing execution keypair in test config")
            .public_key();

        PersistentSafetyStorage::initialize(
            internal_storage,
            author,
            consensus_private_key,
            Some(execution_public_key),
            waypoint,
        )
    } else {
        PersistentSafetyStorage::new(internal_storage)
    }
}

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

        let storage = storage(config);
        let sr_config = &config.consensus.safety_rules;
        match sr_config.service {
            SafetyRulesService::Local => Self::new_local(storage),
            SafetyRulesService::Serializer => Self::new_serializer(storage),
            SafetyRulesService::Thread => Self::new_thread(storage),
            _ => panic!("Unimplemented SafetyRulesService: {:?}", sr_config.service),
        }
    }

    pub fn new_local(storage: PersistentSafetyStorage) -> Self {
        let safety_rules = SafetyRules::new(storage);
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

    pub fn new_serializer(storage: PersistentSafetyStorage) -> Self {
        let safety_rules = SafetyRules::new(storage);
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

    pub fn new_thread(storage: PersistentSafetyStorage) -> Self {
        let thread = ThreadService::new(storage);
        Self {
            internal_safety_rules: SafetyRulesWrapper::Thread(thread),
        }
    }

    pub fn client(&self) -> Box<dyn TSafetyRules + Send + Sync> {
        match &self.internal_safety_rules {
            SafetyRulesWrapper::Local(safety_rules) => {
                Box::new(LocalClient::new(safety_rules.clone()))
            }
            SafetyRulesWrapper::Process(process) => Box::new(process.client()),
            SafetyRulesWrapper::Serializer(serializer_service) => {
                Box::new(SerializerClient::new(serializer_service.clone()))
            }
            SafetyRulesWrapper::SpawnedProcess(process) => Box::new(process.client()),
            SafetyRulesWrapper::Thread(thread) => Box::new(thread.client()),
        }
    }
}
