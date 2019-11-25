// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::keys::KeyPair;
use libra_crypto::{ed25519::Ed25519PrivateKey, Uniform};
use libra_tools::tempdir::TempPath;
use rand::rngs::StdRng;
use serde::{Deserialize, Serialize};
use std::path::Path;

type AccountKeyPair = KeyPair<Ed25519PrivateKey>;

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct TestConfig {
    pub account_keypair: Option<AccountKeyPair>,
    // Used only to prevent a potentially temporary data_dir from being deleted. This should
    // eventually be moved to be owned by something outside the config.
    #[serde(skip)]
    temp_dir: Option<TempPath>,
}

impl Clone for TestConfig {
    fn clone(&self) -> Self {
        Self {
            account_keypair: None,
            temp_dir: None,
        }
    }
}

impl PartialEq for TestConfig {
    fn eq(&self, other: &Self) -> bool {
        self.account_keypair == other.account_keypair
    }
}

impl TestConfig {
    pub fn new_with_temp_dir() -> Self {
        let temp_dir = TempPath::new();
        temp_dir.create_as_dir().expect("error creating tempdir");
        Self {
            account_keypair: None,
            temp_dir: Some(temp_dir),
        }
    }

    pub fn random(&mut self, rng: &mut StdRng) {
        let privkey = Ed25519PrivateKey::generate_for_testing(rng);
        self.account_keypair = Some(AccountKeyPair::load(privkey));
    }

    pub fn temp_dir(&self) -> Option<&Path> {
        if let Some(temp_dir) = self.temp_dir.as_ref() {
            Some(temp_dir.path())
        } else {
            None
        }
    }
}
