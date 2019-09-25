// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::sync::{Arc, RwLock};
use types::crypto_proxies::ValidatorVerifier;

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

    pub fn validators(&self) -> Arc<ValidatorVerifier> {
        Arc::clone(&self.validators.read().unwrap())
    }
}
