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
use consensus_types::common::{Author, Payload};
use libra_config::config::{NodeConfig, SafetyRulesBackend, SafetyRulesService};
use libra_secure_storage::{InMemoryStorage, OnDiskStorage, Storage};
use std::{
    net::SocketAddr,
    sync::{Arc, RwLock},
};

pub fn extract_service_inputs(config: &mut NodeConfig) -> (Author, PersistentSafetyStorage) {
    let author = config
        .validator_network
        .as_ref()
        .expect("Missing validator network")
        .peer_id;

    let backend = &config.consensus.safety_rules.backend;
    let (initialize, internal_storage): (bool, Box<dyn Storage>) = match backend {
        SafetyRulesBackend::InMemoryStorage => (true, Box::new(InMemoryStorage::new())),
        SafetyRulesBackend::OnDiskStorage(config) => {
            (config.default, Box::new(OnDiskStorage::new(config.path())))
        }
    };

    let storage = if initialize {
        let test_config = config.test.as_mut().expect("Missing test config");
        let private_key = test_config
            .consensus_keypair
            .as_mut()
            .expect("Missing consensus keypair in test config")
            .take_private()
            .expect("Failed to take Consensus private key, key absent or already read");

        PersistentSafetyStorage::initialize(internal_storage, private_key)
    } else {
        PersistentSafetyStorage::new(internal_storage)
    };

    (author, storage)
}

enum SafetyRulesWrapper<T> {
    Local(Arc<RwLock<SafetyRules<T>>>),
    Process(ProcessService<T>),
    Serializer(Arc<RwLock<SerializerService<T>>>),
    SpawnedProcess(SpawnedProcess<T>),
    Thread(ThreadService<T>),
}

pub struct SafetyRulesManager<T> {
    internal_safety_rules: SafetyRulesWrapper<T>,
}

impl<T: Payload> SafetyRulesManager<T> {
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
        let process_service = ProcessService::<T>::new(server_addr);
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
        let process = SpawnedProcess::<T>::new(config);
        Self {
            internal_safety_rules: SafetyRulesWrapper::SpawnedProcess(process),
        }
    }

    pub fn new_thread(author: Author, storage: PersistentSafetyStorage) -> Self {
        let thread = ThreadService::<T>::new(author, storage);
        Self {
            internal_safety_rules: SafetyRulesWrapper::Thread(thread),
        }
    }

    pub fn client(&self) -> Box<dyn TSafetyRules<T> + Send + Sync> {
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
