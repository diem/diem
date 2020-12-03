// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::FuzzTargetImpl;
use diem_json_rpc::{
    fuzzing::{fuzzer, generate_corpus, method_fuzzer},
    gen_request_params,
};
use diem_proptest_helpers::ValueGenerator;

const ADDRESS: &str = "000000000000000000000000000000dd";

#[derive(Clone, Debug, Default)]
pub struct JsonRpcSubmitTransactionRequest;

impl FuzzTargetImpl for JsonRpcSubmitTransactionRequest {
    fn description(&self) -> &'static str {
        "JSON RPC submit transaction request"
    }

    fn generate(&self, _idx: usize, gen: &mut ValueGenerator) -> Option<Vec<u8>> {
        Some(generate_corpus(gen))
    }

    fn fuzz(&self, data: &[u8]) {
        fuzzer(data);
    }
}

#[derive(Clone, Debug, Default)]
pub struct JsonRpcGetAccountRequest;

impl FuzzTargetImpl for JsonRpcGetAccountRequest {
    fn description(&self) -> &'static str {
        "JSON RPC get account request"
    }

    fn generate(&self, _idx: usize, _gen: &mut ValueGenerator) -> Option<Vec<u8>> {
        Some(gen_request_params!([ADDRESS]))
    }

    fn fuzz(&self, data: &[u8]) {
        method_fuzzer(data, "get_account");
    }
}

#[derive(Clone, Debug, Default)]
pub struct JsonRpcGetAccountTransactionRequest;

impl FuzzTargetImpl for JsonRpcGetAccountTransactionRequest {
    fn description(&self) -> &'static str {
        "JSON RPC get_account_transaction request"
    }

    fn generate(&self, _idx: usize, _gen: &mut ValueGenerator) -> Option<Vec<u8>> {
        Some(gen_request_params!([ADDRESS, 0, true]))
    }

    fn fuzz(&self, data: &[u8]) {
        method_fuzzer(data, "get_account_transaction");
    }
}

#[derive(Clone, Debug, Default)]
pub struct JsonRpcGetAccountTransactionsRequest;

impl FuzzTargetImpl for JsonRpcGetAccountTransactionsRequest {
    fn description(&self) -> &'static str {
        "JSON RPC get_account_transactions request"
    }

    fn generate(&self, _idx: usize, _gen: &mut ValueGenerator) -> Option<Vec<u8>> {
        Some(gen_request_params!([ADDRESS, 0, 1, true]))
    }

    fn fuzz(&self, data: &[u8]) {
        method_fuzzer(data, "get_account_transactions");
    }
}

#[derive(Clone, Debug, Default)]
pub struct JsonRpcGetTransactionsRequest;

impl FuzzTargetImpl for JsonRpcGetTransactionsRequest {
    fn description(&self) -> &'static str {
        "JSON RPC get_transactions request"
    }

    fn generate(&self, _idx: usize, _gen: &mut ValueGenerator) -> Option<Vec<u8>> {
        Some(gen_request_params!([0, 1, true]))
    }

    fn fuzz(&self, data: &[u8]) {
        method_fuzzer(data, "get_transactions");
    }
}

#[derive(Clone, Debug, Default)]
pub struct JsonRpcGetEventsRequest;

impl FuzzTargetImpl for JsonRpcGetEventsRequest {
    fn description(&self) -> &'static str {
        "JSON RPC get_events request"
    }

    fn generate(&self, _idx: usize, _gen: &mut ValueGenerator) -> Option<Vec<u8>> {
        Some(gen_request_params!([
            "00000000000000000000000000000000000000000a550c18",
            0,
            10
        ]))
    }

    fn fuzz(&self, data: &[u8]) {
        method_fuzzer(data, "get_events");
    }
}

#[derive(Clone, Debug, Default)]
pub struct JsonRpcGetMetadataRequest;

impl FuzzTargetImpl for JsonRpcGetMetadataRequest {
    fn description(&self) -> &'static str {
        "JSON RPC get_metadata request"
    }

    fn generate(&self, _idx: usize, _gen: &mut ValueGenerator) -> Option<Vec<u8>> {
        Some(gen_request_params!([0]))
    }

    fn fuzz(&self, data: &[u8]) {
        method_fuzzer(data, "get_metadata");
    }
}

#[derive(Clone, Debug, Default)]
pub struct JsonRpcGetCurrenciesRequest;

impl FuzzTargetImpl for JsonRpcGetCurrenciesRequest {
    fn description(&self) -> &'static str {
        "JSON RPC get_currencies request"
    }

    fn generate(&self, _idx: usize, _gen: &mut ValueGenerator) -> Option<Vec<u8>> {
        Some(gen_request_params!([]))
    }

    fn fuzz(&self, data: &[u8]) {
        method_fuzzer(data, "get_currencies");
    }
}

#[derive(Clone, Debug, Default)]
pub struct JsonRpcGetStateProofRequest;

impl FuzzTargetImpl for JsonRpcGetStateProofRequest {
    fn description(&self) -> &'static str {
        "JSON RPC get_state_proof request"
    }

    fn generate(&self, _idx: usize, _gen: &mut ValueGenerator) -> Option<Vec<u8>> {
        Some(gen_request_params!([1]))
    }

    fn fuzz(&self, data: &[u8]) {
        method_fuzzer(data, "get_state_proof");
    }
}

#[derive(Clone, Debug, Default)]
pub struct JsonRpcGetAccountStateWithProofRequest;

impl FuzzTargetImpl for JsonRpcGetAccountStateWithProofRequest {
    fn description(&self) -> &'static str {
        "JSON RPC get_account_state_with_proof request"
    }

    fn generate(&self, _idx: usize, _gen: &mut ValueGenerator) -> Option<Vec<u8>> {
        Some(gen_request_params!([ADDRESS, 0, 1]))
    }

    fn fuzz(&self, data: &[u8]) {
        method_fuzzer(data, "get_account_state_with_proof");
    }
}

#[derive(Clone, Debug, Default)]
pub struct JsonRpcGetNetworkStatusRequest;

impl FuzzTargetImpl for JsonRpcGetNetworkStatusRequest {
    fn description(&self) -> &'static str {
        "JSON RPC get_network_status request"
    }

    fn generate(&self, _idx: usize, _gen: &mut ValueGenerator) -> Option<Vec<u8>> {
        Some(gen_request_params!([]))
    }

    fn fuzz(&self, data: &[u8]) {
        method_fuzzer(data, "get_network_status");
    }
}
