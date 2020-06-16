// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::safety_rules_client::{test_utils, tests::suite};
use libra_types::validator_signer::ValidatorSigner;
use safety_rules::{SafetyRules, TSafetyRules};

#[test]
fn test() {
    suite::run_test_suite(safety_rules);
}

fn safety_rules() -> (Box<dyn TSafetyRules>, ValidatorSigner) {
    let signer = ValidatorSigner::from_int(0);
    let storage = test_utils::test_storage(&signer);
    let safety_rules = Box::new(SafetyRules::new(signer.author(), storage));
    (safety_rules, signer)
}
