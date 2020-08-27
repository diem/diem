// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{corpus_from_strategy, fuzz_data_to_value, FuzzTargetImpl};
use libra_proptest_helpers::ValueGenerator;
use safety_rules::{
    fuzz_handle_message, fuzz_initialize,
    fuzzing_utils::{arb_initialize_input, arb_safety_rules_input},
};

#[derive(Clone, Debug, Default)]
pub struct SafetyRulesHandleMessage;

/// This implementation will fuzz the handle_message() method of the safety rules serializer
/// service.
impl FuzzTargetImpl for SafetyRulesHandleMessage {
    fn description(&self) -> &'static str {
        "Safety rules: handle_message()"
    }

    fn generate(&self, _idx: usize, _gen: &mut ValueGenerator) -> Option<Vec<u8>> {
        Some(corpus_from_strategy(arb_safety_rules_input()))
    }

    fn fuzz(&self, data: &[u8]) {
        let safety_rules_input = fuzz_data_to_value(data, arb_safety_rules_input());
        let _ = fuzz_handle_message(safety_rules_input);
    }
}

#[derive(Clone, Debug, Default)]
pub struct SafetyRulesInitialize;

/// This implementation will fuzz the initialize() method of safety rules.
impl FuzzTargetImpl for SafetyRulesInitialize {
    fn description(&self) -> &'static str {
        "Safety rules: initialize()"
    }

    fn generate(&self, _idx: usize, _gen: &mut ValueGenerator) -> Option<Vec<u8>> {
        Some(corpus_from_strategy(arb_initialize_input()))
    }

    fn fuzz(&self, data: &[u8]) {
        let proof = fuzz_data_to_value(data, arb_initialize_input());
        let _ = fuzz_initialize(proof);
    }
}
