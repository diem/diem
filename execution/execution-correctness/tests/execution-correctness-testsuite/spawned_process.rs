// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::suite;
use execution_correctness::{process_client_wrapper::ProcessClientWrapper, ExecutionCorrectness};
use executor_test_helpers::start_storage_service;
use libra_crypto::ed25519::Ed25519PublicKey;

// const BINARY: &str = "execution-correctness";
const BINARY: &str = env!("CARGO_BIN_EXE_execution-correctness");

#[test]
fn test() {
    suite::run_test_suite(execution_correctness());
}

fn execution_correctness() -> (Box<dyn ExecutionCorrectness>, Option<Ed25519PublicKey>) {
    let (config, _handle, _db) = start_storage_service();
    let client_wrapper = ProcessClientWrapper::new(BINARY, config.storage.address);
    let pubkey = client_wrapper.pubkey();
    (Box::new(client_wrapper), pubkey)
}
