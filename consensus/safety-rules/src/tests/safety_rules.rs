// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{tests::suite, PersistentStorage, SafetyRules, TSafetyRules};
use consensus_types::common::{Payload, Round};
use libra_types::crypto_proxies::ValidatorSigner;

#[test]
fn test() {
    suite::run_test_suite(safety_rules::<Round>, safety_rules::<Vec<u8>>);
}

fn safety_rules<T: Payload>() -> (Box<dyn TSafetyRules<T>>, ValidatorSigner) {
    let signer = ValidatorSigner::from_int(0);
    let storage = PersistentStorage::in_memory(signer.private_key().clone());
    let safety_rules = Box::new(SafetyRules::<T>::new(signer.author(), storage));
    (safety_rules, signer)
}
