// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::keys::ConfigKey;
use diem_crypto::{ed25519::Ed25519PrivateKey, PrivateKey, Uniform};
use diem_temppath::TempPath;
use diem_types::{
    on_chain_config::VMPublishingOption, transaction::authenticator::AuthenticationKey,
};
use rand::rngs::StdRng;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TestConfig {
    pub auth_key: Option<AuthenticationKey>,
    pub operator_key: Option<ConfigKey<Ed25519PrivateKey>>,
    pub owner_key: Option<ConfigKey<Ed25519PrivateKey>>,
    pub execution_key: Option<ConfigKey<Ed25519PrivateKey>>,
    // Used only to prevent a potentially temporary data_dir from being deleted. This should
    // eventually be moved to be owned by something outside the config.
    #[serde(skip)]
    temp_dir: Option<TempPath>,

    pub publishing_option: Option<VMPublishingOption>,
}

impl Clone for TestConfig {
    fn clone(&self) -> Self {
        Self {
            auth_key: self.auth_key,
            operator_key: self.operator_key.clone(),
            owner_key: self.owner_key.clone(),
            execution_key: self.execution_key.clone(),
            temp_dir: None,
            publishing_option: self.publishing_option.clone(),
        }
    }
}

impl PartialEq for TestConfig {
    fn eq(&self, other: &Self) -> bool {
        self.operator_key == other.operator_key
            && self.owner_key == other.owner_key
            && self.auth_key == other.auth_key
            && self.execution_key == other.execution_key
    }
}

impl TestConfig {
    pub fn open_module() -> Self {
        Self {
            auth_key: None,
            operator_key: None,
            owner_key: None,
            execution_key: None,
            temp_dir: None,
            publishing_option: Some(VMPublishingOption::open()),
        }
    }

    pub fn new_with_temp_dir(temp_dir: Option<TempPath>) -> Self {
        let temp_dir = temp_dir.unwrap_or_else(|| {
            let temp_dir = TempPath::new();
            temp_dir.create_as_dir().expect("error creating tempdir");
            temp_dir
        });
        Self {
            auth_key: None,
            operator_key: None,
            owner_key: None,
            execution_key: None,
            temp_dir: Some(temp_dir),
            publishing_option: None,
        }
    }

    pub fn execution_key(&mut self, key: Ed25519PrivateKey) {
        self.execution_key = Some(ConfigKey::new(key))
    }

    pub fn operator_key(&mut self, key: Ed25519PrivateKey) {
        self.operator_key = Some(ConfigKey::new(key))
    }

    pub fn owner_key(&mut self, key: Ed25519PrivateKey) {
        self.owner_key = Some(ConfigKey::new(key))
    }

    pub fn random_account_key(&mut self, rng: &mut StdRng) {
        let privkey = Ed25519PrivateKey::generate(rng);
        self.auth_key = Some(AuthenticationKey::ed25519(&privkey.public_key()));
        self.operator_key = Some(ConfigKey::new(privkey));

        let privkey = Ed25519PrivateKey::generate(rng);
        self.owner_key = Some(ConfigKey::new(privkey));
    }

    pub fn random_execution_key(&mut self, rng: &mut StdRng) {
        let privkey = Ed25519PrivateKey::generate(rng);
        self.execution_key = Some(ConfigKey::new(privkey));
    }

    pub fn temp_dir(&self) -> Option<&Path> {
        self.temp_dir.as_ref().map(|temp_dir| temp_dir.path())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rand::{rngs::StdRng, SeedableRng};

    #[test]
    fn verify_test_config_equality_using_keys() {
        // Create default test config without keys
        let mut test_config = TestConfig::new_with_temp_dir(None);
        assert_eq!(test_config.operator_key, None);
        assert_eq!(test_config.owner_key, None);
        assert_eq!(test_config.execution_key, None);

        // Clone the config and verify equality
        let mut clone_test_config = test_config.clone();
        assert_eq!(clone_test_config, test_config);

        // Generate keys for original test config
        let mut rng = StdRng::from_seed([0u8; 32]);
        test_config.random_account_key(&mut rng);
        test_config.random_execution_key(&mut rng);

        // Verify that configs differ
        assert_ne!(clone_test_config, test_config);

        // Copy keys across configs
        clone_test_config.auth_key = test_config.auth_key;
        clone_test_config.execution_key = test_config.execution_key.clone();
        clone_test_config.operator_key = test_config.operator_key.clone();
        clone_test_config.owner_key = test_config.owner_key.clone();

        // Verify both configs are identical
        assert_eq!(clone_test_config, test_config);
    }
}
