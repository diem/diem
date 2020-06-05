// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{test_utils, tests::suite, SafetyRules, TSafetyRules};
use libra_crypto::{ed25519::Ed25519PrivateKey, Uniform};
use libra_types::validator_signer::ValidatorSigner;

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
    let storage = test_utils::test_storage(&signer);
    let safety_rules = Box::new(SafetyRules::new(storage, verify_vote_proposal_signature));
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
