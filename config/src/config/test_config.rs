// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::keys::KeyPair;
use libra_crypto::{ed25519::Ed25519PrivateKey, PrivateKey, Uniform};
use libra_temppath::TempPath;
use libra_types::{
    on_chain_config::VMPublishingOption, transaction::authenticator::AuthenticationKey,
};
use rand::rngs::StdRng;
use serde::{Deserialize, Serialize};
use std::path::Path;

type AccountKeyPair = KeyPair<Ed25519PrivateKey>;
type ExecutionKeyPair = KeyPair<Ed25519PrivateKey>;

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TestConfig {
    pub auth_key: Option<AuthenticationKey>,
    #[serde(rename = "operator_private_key")]
    pub operator_keypair: Option<AccountKeyPair>,
    #[serde(rename = "execution_private_key")]
    pub execution_keypair: Option<ExecutionKeyPair>,
    // Used only to prevent a potentially temporary data_dir from being deleted. This should
    // eventually be moved to be owned by something outside the config.
    #[serde(skip)]
    temp_dir: Option<TempPath>,

    pub publishing_option: Option<VMPublishingOption>,
}

#[cfg(any(test, feature = "fuzzing"))]
impl Clone for TestConfig {
    fn clone(&self) -> Self {
        Self {
            auth_key: self.auth_key,
            operator_keypair: self.operator_keypair.clone(),
            execution_keypair: self.execution_keypair.clone(),
            temp_dir: None,
            publishing_option: self.publishing_option.clone(),
        }
    }
}

impl PartialEq for TestConfig {
    fn eq(&self, other: &Self) -> bool {
        self.operator_keypair == other.operator_keypair
            && self.auth_key == other.auth_key
            && self.execution_keypair == other.execution_keypair
    }
}

impl TestConfig {
    pub fn open_module() -> Self {
        Self {
            auth_key: None,
            operator_keypair: None,
            execution_keypair: None,
            temp_dir: None,
            publishing_option: Some(VMPublishingOption::Open),
        }
    }

    pub fn new_with_temp_dir() -> Self {
        let temp_dir = TempPath::new();
        temp_dir.create_as_dir().expect("error creating tempdir");
        Self {
            auth_key: None,
            operator_keypair: None,
            execution_keypair: None,
            temp_dir: Some(temp_dir),
            publishing_option: None,
        }
    }

    pub fn random_account_key(&mut self, rng: &mut StdRng) {
        let privkey = Ed25519PrivateKey::generate(rng);
        self.auth_key = Some(AuthenticationKey::ed25519(&privkey.public_key()));
        self.operator_keypair = Some(AccountKeyPair::load(privkey));
    }

    pub fn random_execution_key(&mut self, rng: &mut StdRng) {
        let privkey = Ed25519PrivateKey::generate(rng);
        self.execution_keypair = Some(ExecutionKeyPair::load(privkey));
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
        let mut test_config = TestConfig::new_with_temp_dir();
        assert_eq!(test_config.operator_keypair, None);
        assert_eq!(test_config.execution_keypair, None);

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
        clone_test_config.execution_keypair = test_config.execution_keypair.clone();
        clone_test_config.operator_keypair = test_config.operator_keypair.clone();

        // Verify both configs are identical
        assert_eq!(clone_test_config, test_config);
    }
}
