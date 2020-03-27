// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::FuzzTargetImpl;
use libra_proptest_helpers::ValueGenerator;
use noise::noise_fuzzing::{fuzz_responder, generate_corpus};

#[derive(Clone, Debug, Default)]
pub struct NetworkNoiseResponder;

impl FuzzTargetImpl for NetworkNoiseResponder {
    fn name(&self) -> &'static str {
        module_name!()
    }

    fn description(&self) -> &'static str {
        "Network Noise crate initiator side"
    }

    fn generate(&self, _idx: usize, gen: &mut ValueGenerator) -> Option<Vec<u8>> {
        Some(generate_corpus(gen))
    }

    fn fuzz(&self, data: &[u8]) {
        fuzz_responder(data);
    }
}
