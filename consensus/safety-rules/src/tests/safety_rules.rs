// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{test_utils, tests::suite, SafetyRules, TSafetyRules};
use libra_types::validator_signer::ValidatorSigner;

#[test]
fn test() {
    suite::run_test_suite(safety_rules);
}

fn safety_rules() -> (Box<dyn TSafetyRules>, ValidatorSigner) {
    let signer = ValidatorSigner::from_int(0);
    let storage = test_utils::test_storage(&signer);
    let safety_rules = Box::new(SafetyRules::new(storage));
    (safety_rules, signer)
}
