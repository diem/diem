// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    local_client::LocalClient, persistent_storage::PersistentStorage, InMemoryStorage,
    OnDiskStorage, SafetyRules, TSafetyRules,
};
use consensus_types::common::Payload;
use libra_config::config::{NodeConfig, SafetyRulesBackend, SafetyRulesConfig};
use libra_types::crypto_proxies::ValidatorSigner;
use std::sync::{Arc, RwLock};

pub struct SafetyRulesManagerConfig {
    storage: Option<Box<dyn PersistentStorage>>,
    validator_signer: Option<ValidatorSigner>,
}

impl SafetyRulesManagerConfig {
    pub fn new(config: &mut NodeConfig) -> Self {
        let private_key = config
            .consensus
            .consensus_keypair
            .take_private()
            .expect("Failed to take Consensus private key, key absent or already read");

        let author = config
            .validator_network
            .as_ref()
            .expect("Missing validator network")
            .peer_id;

        let validator_signer = ValidatorSigner::new(author, private_key);
        Self::new_with_signer(validator_signer, &config.consensus.safety_rules)
    }

    pub fn new_with_signer(validator_signer: ValidatorSigner, config: &SafetyRulesConfig) -> Self {
        let storage = match &config.backend {
            SafetyRulesBackend::InMemoryStorage => InMemoryStorage::default_storage(),
            SafetyRulesBackend::OnDiskStorage(config) => {
                if config.default {
                    OnDiskStorage::default_storage(config.path().clone())
                        .expect("Unable to allocate SafetyRules storage")
                } else {
                    OnDiskStorage::new_storage(config.path().clone())
                        .expect("Unable to instantiate SafetyRules storage")
                }
            }
        };

        Self {
            storage: Some(storage),
            validator_signer: Some(validator_signer),
        }
    }
}

enum SafetyRulesWrapper<T> {
    Local(Arc<RwLock<SafetyRules<T>>>),
}

pub struct SafetyRulesManager<T> {
    internal_safety_rules: SafetyRulesWrapper<T>,
}

impl<T: Payload> SafetyRulesManager<T> {
    pub fn new(mut config: SafetyRulesManagerConfig) -> Self {
        let storage = config.storage.take().expect("validator_signer missing");
        let validator_signer = config
            .validator_signer
            .take()
            .expect("validator_signer missing");
        Self::new_direct(storage, validator_signer)
    }

    pub fn new_direct(
        storage: Box<dyn PersistentStorage>,
        validator_signer: ValidatorSigner,
    ) -> Self {
        let safety_rules = SafetyRules::new(storage, Arc::new(validator_signer));

        Self {
            internal_safety_rules: SafetyRulesWrapper::Local(Arc::new(RwLock::new(safety_rules))),
        }
    }

    pub fn client(&self) -> Box<dyn TSafetyRules<T> + Send + Sync> {
        match &self.internal_safety_rules {
            SafetyRulesWrapper::Local(safety_rules) => {
                Box::new(LocalClient::new(safety_rules.clone()))
            }
        }
    }
}
