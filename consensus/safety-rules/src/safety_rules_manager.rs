// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    local_client::LocalClient,
    persistent_storage::PersistentStorage,
    remote_service::RemoteService,
    serializer::{SerializerClient, SerializerService},
    thread::ThreadService,
    InMemoryStorage, OnDiskStorage, SafetyRules, TSafetyRules,
};
use consensus_types::common::Payload;
use libra_config::config::{NodeConfig, SafetyRulesBackend, SafetyRulesService};
use libra_types::crypto_proxies::ValidatorSigner;
use std::sync::{Arc, RwLock};

pub fn extract_service_inputs(
    config: &mut NodeConfig,
) -> (ValidatorSigner, Box<dyn PersistentStorage>) {
    let private_key = config
        .test
        .as_mut()
        .expect("Missing test config")
        .consensus_keypair
        .as_mut()
        .expect("Missing consensus keypair in test config")
        .take_private()
        .expect("Failed to take Consensus private key, key absent or already read");

    let author = config
        .validator_network
        .as_ref()
        .expect("Missing validator network")
        .peer_id;

    let validator_signer = ValidatorSigner::new(author, private_key);

    let storage = match &config.consensus.safety_rules.backend {
        SafetyRulesBackend::InMemoryStorage => InMemoryStorage::default_storage(),
        SafetyRulesBackend::OnDiskStorage(config) => {
            if config.default {
                OnDiskStorage::default_storage(config.path())
                    .expect("Unable to allocate SafetyRules storage")
            } else {
                OnDiskStorage::new_storage(config.path())
                    .expect("Unable to instantiate SafetyRules storage")
            }
        }
    };

    (validator_signer, storage)
}

enum SafetyRulesWrapper<T> {
    Local(Arc<RwLock<SafetyRules<T>>>),
    Serializer(Arc<RwLock<SerializerService<T>>>),
    Thread(ThreadService<T>),
}

pub struct SafetyRulesManager<T> {
    internal_safety_rules: SafetyRulesWrapper<T>,
}

impl<T: Payload> SafetyRulesManager<T> {
    pub fn new(config: &mut NodeConfig) -> Self {
        let (validator_signer, storage) = extract_service_inputs(config);
        let sr_config = &config.consensus.safety_rules;
        match sr_config.service {
            SafetyRulesService::Local => Self::new_local(storage, validator_signer),
            SafetyRulesService::Serializer => Self::new_serializer(storage, validator_signer),
            SafetyRulesService::Thread => Self::new_thread(storage, validator_signer),
            _ => panic!("Unimplemented SafetyRulesService: {:?}", sr_config.service),
        }
    }

    pub fn new_local(
        storage: Box<dyn PersistentStorage>,
        validator_signer: ValidatorSigner,
    ) -> Self {
        let safety_rules = SafetyRules::new(storage, Arc::new(validator_signer));

        Self {
            internal_safety_rules: SafetyRulesWrapper::Local(Arc::new(RwLock::new(safety_rules))),
        }
    }

    pub fn new_serializer(
        storage: Box<dyn PersistentStorage>,
        validator_signer: ValidatorSigner,
    ) -> Self {
        let safety_rules = SafetyRules::new(storage, Arc::new(validator_signer));
        let serializer_service = SerializerService::new(safety_rules);

        Self {
            internal_safety_rules: SafetyRulesWrapper::Serializer(Arc::new(RwLock::new(
                serializer_service,
            ))),
        }
    }

    pub fn new_thread(
        storage: Box<dyn PersistentStorage>,
        validator_signer: ValidatorSigner,
    ) -> Self {
        let thread = ThreadService::<T>::new(storage, validator_signer);

        Self {
            internal_safety_rules: SafetyRulesWrapper::Thread(thread),
        }
    }

    pub fn client(&self) -> Box<dyn TSafetyRules<T> + Send + Sync> {
        match &self.internal_safety_rules {
            SafetyRulesWrapper::Local(safety_rules) => {
                Box::new(LocalClient::new(safety_rules.clone()))
            }
            SafetyRulesWrapper::Serializer(serializer_service) => {
                Box::new(SerializerClient::new(serializer_service.clone()))
            }
            SafetyRulesWrapper::Thread(thread) => Box::new(thread.client()),
            _ => panic!("Unable to retrieve client for this service"),
        }
    }
}
