// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::FuzzTargetImpl;
use libra_proptest_helpers::ValueGenerator;
use network::protocols::rpc;

#[derive(Clone, Debug, Default)]
pub struct RpcInboundRequest;

impl FuzzTargetImpl for RpcInboundRequest {
    fn name(&self) -> &'static str {
        module_name!()
    }

    fn description(&self) -> &'static str {
        "P2P Network Inbound RPC Request"
    }

    fn generate(&self, _idx: usize, gen: &mut ValueGenerator) -> Option<Vec<u8>> {
        Some(rpc::fuzzing::generate_corpus(gen))
    }

    fn fuzz(&self, data: &[u8]) {
        rpc::fuzzing::fuzzer(data);
    }
}
