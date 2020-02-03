// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::keys::KeyPair;
use libra_crypto::{ed25519::Ed25519PrivateKey, Uniform};
use libra_temppath::TempPath;
use rand::rngs::StdRng;
use serde::{Deserialize, Serialize};
use std::path::Path;

type AccountKeyPair = KeyPair<Ed25519PrivateKey>;
type ConsensusKeyPair = KeyPair<Ed25519PrivateKey>;

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct TestConfig {
    pub account_keypair: Option<AccountKeyPair>,
    // Used only to prevent a potentially temporary data_dir from being deleted. This should
    // eventually be moved to be owned by something outside the config.
    pub consensus_keypair: Option<ConsensusKeyPair>,
    #[serde(skip)]
    temp_dir: Option<TempPath>,
}

#[cfg(any(test, feature = "fuzzing"))]
impl Clone for TestConfig {
    fn clone(&self) -> Self {
        Self {
            account_keypair: self.account_keypair.clone(),
            consensus_keypair: self.consensus_keypair.clone(),
            temp_dir: None,
        }
    }
}

impl PartialEq for TestConfig {
    fn eq(&self, other: &Self) -> bool {
        self.account_keypair == other.account_keypair
            && self.consensus_keypair == other.consensus_keypair
    }
}

impl TestConfig {
    pub fn new_with_temp_dir() -> Self {
        let temp_dir = TempPath::new();
        temp_dir.create_as_dir().expect("error creating tempdir");
        Self {
            account_keypair: None,
            consensus_keypair: None,
            temp_dir: Some(temp_dir),
        }
    }

    pub fn random_account_key(&mut self, rng: &mut StdRng) {
        let privkey = Ed25519PrivateKey::generate_for_testing(rng);
        self.account_keypair = Some(AccountKeyPair::load(privkey));
    }

    pub fn random_consensus_key(&mut self, rng: &mut StdRng) {
        let privkey = Ed25519PrivateKey::generate_for_testing(rng);
        self.consensus_keypair = Some(ConsensusKeyPair::load(privkey));
    }

    pub fn temp_dir(&self) -> Option<&Path> {
        if let Some(temp_dir) = self.temp_dir.as_ref() {
            Some(temp_dir.path())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rand::{rngs::StdRng, SeedableRng};

    #[test]
    fn verify_test_config_equality_using_keys() {
        // Create default test config without keys
        let mut test_config = TestConfig::new_with_temp_dir();
        assert_eq!(test_config.account_keypair, None);
        assert_eq!(test_config.consensus_keypair, None);

        // Clone the config and verify equality
        let mut clone_test_config = test_config.clone();
        assert_eq!(clone_test_config, test_config);

        // Generate keys for original test config
        let mut rng = StdRng::from_seed([0u8; 32]);
        test_config.random_account_key(&mut rng);
        test_config.random_consensus_key(&mut rng);

        // Verify that configs differ
        assert_ne!(clone_test_config, test_config);

        // Copy keys across configs
        clone_test_config.account_keypair = test_config.account_keypair.clone();
        clone_test_config.consensus_keypair = test_config.consensus_keypair.clone();

        // Verify both configs are identical
        assert_eq!(clone_test_config, test_config);
    }
}
