// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{tests::suite, PersistentStorage, SafetyRulesManager, TSafetyRules};
use consensus_types::common::{Payload, Round};
use libra_types::crypto_proxies::ValidatorSigner;
use std::sync::Arc;

#[test]
fn test() {
    suite::run_test_suite(safety_rules::<Round>, safety_rules::<Vec<u8>>);
}

fn safety_rules<T: Payload>() -> (Box<dyn TSafetyRules<T>>, Arc<ValidatorSigner>) {
    let signer = ValidatorSigner::from_int(0);
    let safety_rules_manager =
        SafetyRulesManager::new_thread(PersistentStorage::in_memory(), signer.clone());
    let safety_rules = safety_rules_manager.client();
    (safety_rules, Arc::new(signer))
}
