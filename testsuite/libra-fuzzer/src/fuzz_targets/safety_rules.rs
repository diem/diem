// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{corpus_from_strategy, fuzz_data_to_value, FuzzTargetImpl};
use libra_proptest_helpers::ValueGenerator;
use safety_rules::{fuzzing_utils::arb_handle_message_input, test_utils};

#[derive(Clone, Debug, Default)]
pub struct SafetyRulesHandleMessage;

/// This implementation will fuzz the handle_message() method of the safety rules serializer
/// service.
impl FuzzTargetImpl for SafetyRulesHandleMessage {
    fn description(&self) -> &'static str {
        "Safety rules: handle_message()"
    }

    fn generate(&self, _idx: usize, _gen: &mut ValueGenerator) -> Option<Vec<u8>> {
        Some(corpus_from_strategy(arb_handle_message_input()))
    }

    fn fuzz(&self, data: &[u8]) {
        // Generate the fuzzer input
        let safety_rules_input = fuzz_data_to_value(data, arb_handle_message_input());

        // Create a safety rules serializer test instance for fuzzing
        let mut serializer_service = test_utils::test_serializer();

        // LCS encode the safety_rules_input and fuzz the handle_message() method
        if let Ok(safety_rules_input) = lcs::to_bytes(&safety_rules_input) {
            let _ = serializer_service.handle_message(safety_rules_input);
        };
    }
}
