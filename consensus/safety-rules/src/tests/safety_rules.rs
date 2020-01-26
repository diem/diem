// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{tests::suite, PersistentStorage, SafetyRules, TSafetyRules};
use consensus_types::common::{Payload, Round};
use libra_types::crypto_proxies::ValidatorSigner;
use std::sync::Arc;

#[test]
fn test() {
    suite::run_test_suite(safety_rules::<Round>, safety_rules::<Vec<u8>>);
}

fn safety_rules<T: Payload>() -> (Box<dyn TSafetyRules<T>>, Arc<ValidatorSigner>) {
    let signer = Arc::new(ValidatorSigner::from_int(0));
    let safety_rules = Box::new(SafetyRules::<T>::new(
        PersistentStorage::in_memory(),
        signer.clone(),
    ));
    (safety_rules, signer)
}
