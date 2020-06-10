// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::start_storage_service;
use crate::{
    execution_correctness::ExecutionCorrectness, tests::suite, ExecutionCorrectnessManager,
};
use libra_crypto::{
    ed25519::{Ed25519PrivateKey, Ed25519PublicKey},
    Uniform,
};

#[test]
fn test() {
    suite::run_test_suite(execution_correctness(true));
    suite::run_test_suite(execution_correctness(false));
}

fn execution_correctness(
    enable_signing: bool,
) -> (Box<dyn ExecutionCorrectness>, Option<Ed25519PublicKey>) {
    let (config, _handle) = start_storage_service();
    let (prikey, pubkey) = if enable_signing {
        let prikey = Ed25519PrivateKey::generate_for_testing();
        let pubkey = Ed25519PublicKey::from(&prikey);
        (Some(prikey), Some(pubkey))
    } else {
        (None, None)
    };
    let execution_correctness_manager =
        ExecutionCorrectnessManager::new_serializer(config.storage.address, prikey);
    (execution_correctness_manager.client(), pubkey)
}
