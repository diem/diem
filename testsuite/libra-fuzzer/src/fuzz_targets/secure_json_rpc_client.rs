// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::FuzzTargetImpl;
use libra_proptest_helpers::ValueGenerator;
use libra_secure_json_rpc::fuzzing::{
    fuzz_account_state_response, fuzz_submit_transaction_response, generate_account_state_corpus,
    generate_submit_transaction_corpus,
};

#[derive(Clone, Debug, Default)]
pub struct SecureJsonRpcSubmitTransaction;

/// This implementation will fuzz the submit_transaction() JSON RPC response returned to the
/// secure client.
impl FuzzTargetImpl for SecureJsonRpcSubmitTransaction {
    fn description(&self) -> &'static str {
        "Secure JSON RPC submit_transaction() response"
    }

    fn generate(&self, _idx: usize, _gen: &mut ValueGenerator) -> Option<Vec<u8>> {
        Some(generate_submit_transaction_corpus())
    }

    fn fuzz(&self, data: &[u8]) {
        fuzz_submit_transaction_response(data);
    }
}

#[derive(Clone, Debug, Default)]
pub struct SecureJsonRpcGetAccountState;

/// This implementation will fuzz the get_account_state_with_proof() JSON RPC response returned to
/// the secure client.
impl FuzzTargetImpl for SecureJsonRpcGetAccountState {
    fn description(&self) -> &'static str {
        "Secure JSON RPC get_account_state_with_proof() response"
    }

    fn generate(&self, _idx: usize, _gen: &mut ValueGenerator) -> Option<Vec<u8>> {
        Some(generate_account_state_corpus())
    }

    fn fuzz(&self, data: &[u8]) {
        fuzz_account_state_response(data);
    }
}

#[derive(Clone, Debug, Default)]
pub struct SecureJsonRpcGetAccountTransaction;

/// This implementation will fuzz the get_account_transaction() JSON RPC response returned to the
/// secure client.
impl FuzzTargetImpl for SecureJsonRpcGetAccountTransaction {
    fn description(&self) -> &'static str {
        "Secure JSON RPC get_account_transaction() response"
    }

    fn generate(&self, _idx: usize, _gen: &mut ValueGenerator) -> Option<Vec<u8>> {
        Some(generate_submit_transaction_corpus())
    }

    fn fuzz(&self, data: &[u8]) {
        fuzz_submit_transaction_response(data);
    }
}
