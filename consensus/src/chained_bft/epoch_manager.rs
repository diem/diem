// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_types::crypto_proxies::ValidatorVerifier;
use std::sync::{Arc, RwLock};

/// Manages the current epoch and validator set to provide quorum size/voting power and signature
/// verification.
pub struct EpochManager {
    #[allow(dead_code)]
    epoch: usize,
    validators: RwLock<Arc<ValidatorVerifier>>,
}

impl EpochManager {
    pub fn new(epoch: usize, validators: ValidatorVerifier) -> Self {
        Self {
            epoch,
            validators: RwLock::new(Arc::new(validators)),
        }
    }

    pub fn quorum_size(&self) -> usize {
        self.validators.read().unwrap().quorum_size()
    }

    pub fn validators(&self) -> Arc<ValidatorVerifier> {
        Arc::clone(&self.validators.read().unwrap())
    }
}
