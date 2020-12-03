// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{corpus_from_strategy, fuzz_data_to_value, FuzzTargetImpl};
use diem_proptest_helpers::ValueGenerator;
use diem_secure_json_rpc::{
    fuzzing::{
        arb_account_state_response, arb_submit_transaction_response,
        arb_transaction_status_response,
    },
    process_account_state_response, process_submit_transaction_response,
    process_transaction_status_response,
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
        Some(corpus_from_strategy(arb_submit_transaction_response()))
    }

    fn fuzz(&self, data: &[u8]) {
        let input = fuzz_data_to_value(data, arb_submit_transaction_response());
        let _ = process_submit_transaction_response(input);
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
        Some(corpus_from_strategy(arb_account_state_response()))
    }

    fn fuzz(&self, data: &[u8]) {
        let input = fuzz_data_to_value(data, arb_account_state_response());
        let _ = process_account_state_response(input);
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
        Some(corpus_from_strategy(arb_transaction_status_response()))
    }

    fn fuzz(&self, data: &[u8]) {
        let input = fuzz_data_to_value(data, arb_transaction_status_response());
        let _ = process_transaction_status_response(input);
    }
}
