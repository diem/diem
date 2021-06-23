// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{corpus_from_strategy, fuzz_data_to_value, FuzzTargetImpl};
use diem_proptest_helpers::ValueGenerator;
use safety_rules::fuzzing_utils::{
    arb_block_data, arb_epoch_change_proof, arb_maybe_signed_vote_proposal, arb_safety_rules_input,
    arb_timeout,
    fuzzing::{
        fuzz_construct_and_sign_vote, fuzz_handle_message, fuzz_initialize, fuzz_sign_proposal,
        fuzz_sign_timeout,
    },
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
        Some(corpus_from_strategy(arb_epoch_change_proof()))
    }

    fn fuzz(&self, data: &[u8]) {
        let epoch_change_proof = fuzz_data_to_value(data, arb_epoch_change_proof());
        let _ = fuzz_initialize(epoch_change_proof);
    }
}

#[derive(Clone, Debug, Default)]
pub struct SafetyRulesConstructAndSignVote;

/// This implementation will fuzz the construct_and_sign_vote() method of safety rules.
impl FuzzTargetImpl for SafetyRulesConstructAndSignVote {
    fn description(&self) -> &'static str {
        "Safety rules: construct_and_sign_vote()"
    }

    fn generate(&self, _idx: usize, _gen: &mut ValueGenerator) -> Option<Vec<u8>> {
        Some(corpus_from_strategy(arb_maybe_signed_vote_proposal()))
    }

    fn fuzz(&self, data: &[u8]) {
        let maybe_signed_vote_proposal = fuzz_data_to_value(data, arb_maybe_signed_vote_proposal());
        let _ = fuzz_construct_and_sign_vote(maybe_signed_vote_proposal);
    }
}

#[derive(Clone, Debug, Default)]
pub struct SafetyRulesSignProposal;

/// This implementation will fuzz the sign_proposal() method of safety rules.
impl FuzzTargetImpl for SafetyRulesSignProposal {
    fn description(&self) -> &'static str {
        "Safety rules: sign_proposal()"
    }

    fn generate(&self, _idx: usize, _gen: &mut ValueGenerator) -> Option<Vec<u8>> {
        Some(corpus_from_strategy(arb_block_data()))
    }

    fn fuzz(&self, data: &[u8]) {
        let block_data = fuzz_data_to_value(data, arb_block_data());
        let _ = fuzz_sign_proposal(&block_data);
    }
}

#[derive(Clone, Debug, Default)]
pub struct SafetyRulesSignTimeout;

/// This implementation will fuzz the sign_timeout() method of safety rules.
impl FuzzTargetImpl for SafetyRulesSignTimeout {
    fn description(&self) -> &'static str {
        "Safety rules: sign_timeout()"
    }

    fn generate(&self, _idx: usize, _gen: &mut ValueGenerator) -> Option<Vec<u8>> {
        Some(corpus_from_strategy(arb_timeout()))
    }

    fn fuzz(&self, data: &[u8]) {
        let timeout = fuzz_data_to_value(data, arb_timeout());
        let _ = fuzz_sign_timeout(timeout);
    }
}
