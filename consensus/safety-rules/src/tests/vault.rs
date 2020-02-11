// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{tests::suite, PersistentSafetyStorage, SafetyRulesManager, TSafetyRules};
use consensus_types::common::{Payload, Round};
use libra_secure_storage::VaultStorage;
use libra_types::crypto_proxies::ValidatorSigner;

/// A test for verifying VaultStorage properly supports the SafetyRule backend.  This test
/// depends on running Vault, which can be done by using the provided docker run script in
/// `docker/vault/run.sh`
#[ignore]
#[test]
fn test() {
    suite::run_test_suite(safety_rules::<Round>, safety_rules::<Vec<u8>>);
}

fn safety_rules<T: Payload>() -> (Box<dyn TSafetyRules<T>>, ValidatorSigner) {
    let signer = ValidatorSigner::from_int(0);
    let host = "http://localhost:8200".to_string();
    let token = "root_token".to_string();

    let storage = VaultStorage::new(host, token);
    storage.reset().unwrap();
    let storage =
        PersistentSafetyStorage::initialize(Box::new(storage), signer.private_key().clone());
    let safety_rules_manager = SafetyRulesManager::new_local(signer.author(), storage);
    let safety_rules = safety_rules_manager.client();
    (safety_rules, signer)
}
