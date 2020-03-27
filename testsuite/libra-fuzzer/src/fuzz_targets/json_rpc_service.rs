// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::FuzzTargetImpl;
use libra_json_rpc::fuzzing::{fuzzer, generate_corpus};
use libra_proptest_helpers::ValueGenerator;

#[derive(Clone, Debug, Default)]
pub struct JsonRpcSubmitTransactionRequest;

impl FuzzTargetImpl for JsonRpcSubmitTransactionRequest {
    fn name(&self) -> &'static str {
        module_name!()
    }

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
