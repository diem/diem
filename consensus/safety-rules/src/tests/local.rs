// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{test_utils, tests::suite, SafetyRulesManager};
use libra_crypto::{ed25519::Ed25519PrivateKey, Uniform};
use libra_types::validator_signer::ValidatorSigner;

#[test]
fn test() {
    suite::run_test_suite(&safety_rules(false));
    suite::run_test_suite(&safety_rules(true));
}

fn safety_rules(verify_vote_proposal_signature: bool) -> suite::Callback {
    Box::new(move || {
        let signer = ValidatorSigner::from_int(0);
        let storage = test_utils::test_storage(&signer);
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
    })
}
