// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_types::crypto_proxies::ValidatorVerifier;
use std::sync::{Arc, RwLock};

/// Manages the current epoch and validator set to provide quorum size/voting power and signature
/// verification.
pub struct EpochManager {
    #[allow(dead_code)]
    epoch: u64,
    validators: RwLock<Arc<ValidatorVerifier>>,
}

impl EpochManager {
    pub fn new(epoch: u64, validators: ValidatorVerifier) -> Self {
        Self {
            epoch,
            validators: RwLock::new(Arc::new(validators)),
        }
    }

    pub fn validators(&self) -> Arc<ValidatorVerifier> {
        Arc::clone(&self.validators.read().unwrap())
    }
}
