// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{tests::suite, PersistentSafetyStorage, SafetyRulesManager, TSafetyRules};
use libra_crypto::{ed25519::Ed25519PrivateKey, Uniform};
use libra_secure_storage::{KVStorage, Storage, VaultStorage};
use libra_types::validator_signer::ValidatorSigner;

/// A test for verifying VaultStorage properly supports the SafetyRule backend.  This test
/// depends on running Vault, which can be done by using the provided docker run script in
/// `docker/vault/run.sh`
#[ignore]
#[test]
fn test() {
    suite::run_test_suite(safety_rules, false);
    suite::run_test_suite(safety_rules, true);
}

fn safety_rules(
    verify_vote_proposal_signature: bool,
) -> (
    Box<dyn TSafetyRules>,
    ValidatorSigner,
    Option<Ed25519PrivateKey>,
) {
    let signer = ValidatorSigner::from_int(0);
    let host = "http://localhost:8200".to_string();
    let token = "root_token".to_string();
    let mut storage = Storage::from(VaultStorage::new(host, token, None, None));
    storage.reset_and_clear().unwrap();

    let waypoint = crate::test_utils::validator_signers_to_waypoint(&[&signer]);
    let storage = PersistentSafetyStorage::initialize(
        storage,
        signer.author(),
        signer.private_key().clone(),
        Ed25519PrivateKey::generate_for_testing(),
        waypoint,
    );
    let safety_rules_manager =
        SafetyRulesManager::new_local(storage, verify_vote_proposal_signature);
    let safety_rules = safety_rules_manager.client();
    (
        safety_rules,
        signer,
        if verify_vote_proposal_signature {
            Some(Ed25519PrivateKey::generate_for_testing())
        } else {
            None
        },
    )
}
