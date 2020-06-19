// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{test_utils, tests::suite, SafetyRulesManager, TSafetyRules};
use libra_types::validator_signer::ValidatorSigner;

#[test]
fn test() {
    suite::run_test_suite(safety_rules);
}

fn safety_rules() -> (Box<dyn TSafetyRules>, ValidatorSigner) {
    let signer = ValidatorSigner::from_int(0);
    let storage = test_utils::test_storage(&signer);
    let safety_rules_manager = SafetyRulesManager::new_local(storage);
    let safety_rules = safety_rules_manager.client();
    (safety_rules, signer)
}
