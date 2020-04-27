// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{process_client_wrapper::ProcessClientWrapper, tests::suite, TSafetyRules};
use consensus_types::common::{Payload, Round};
use libra_config::config::SecureBackend;
use libra_types::validator_signer::ValidatorSigner;

#[test]
fn test() {
    suite::run_test_suite(safety_rules::<Round>, safety_rules::<Vec<u8>>);
}

fn safety_rules<T: Payload>() -> (Box<dyn TSafetyRules<T>>, ValidatorSigner) {
    let client_wrapper = ProcessClientWrapper::new(SecureBackend::InMemoryStorage);
    let signer = client_wrapper.signer();
    (Box::new(client_wrapper), signer)
}
