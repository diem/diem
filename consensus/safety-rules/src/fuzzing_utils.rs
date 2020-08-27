// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::serializer::SafetyRulesInput;
use consensus_types::{
    block::Block,
    block_data::{BlockData, BlockType},
    quorum_cert::QuorumCert,
    timeout::Timeout,
    vote_data::VoteData,
    vote_proposal::{MaybeSignedVoteProposal, VoteProposal},
};
use libra_crypto::{
    ed25519::{Ed25519PublicKey, Ed25519Signature},
    hash::{HashValue, TransactionAccumulatorHasher},
};
use libra_types::{
    account_address::AccountAddress,
    epoch_change::EpochChangeProof,
    epoch_state::EpochState,
    ledger_info::LedgerInfoWithSignatures,
    proof::AccumulatorExtensionProof,
    proptest_types::{AccountInfoUniverse, BlockInfoGen},
    transaction::SignedTransaction,
    validator_verifier::{ValidatorConsensusInfo, ValidatorVerifier},
};
use proptest::prelude::*;
use std::convert::TryFrom;

const MAX_BLOCK_SIZE: usize = 10000;
const MAX_NUM_ADDR_TO_VALIDATOR_INFO: usize = 10;
const MAX_NUM_LEAVES: usize = 20;
const MAX_NUM_LEDGER_INFO_WITH_SIGS: usize = 10;
const MAX_NUM_SUBTREE_ROOTS: usize = 20;
const MAX_PROPOSAL_TRANSACTIONS: usize = 5;
const NUM_UNIVERSE_ACCOUNTS: usize = 3;

// This generates an arbitrary AccumulatorExtensionProof<TransactionAccumulatorHasher>.
prop_compose! {
    pub fn arb_accumulator_extension_proof(
    )(
        frozen_subtree_roots in prop::collection::vec(any::<HashValue>(), 0..MAX_NUM_SUBTREE_ROOTS),
        leaf_count in any::<u64>(),
        leaves in prop::collection::vec(any::<HashValue>(), 0..MAX_NUM_LEAVES),
    ) -> AccumulatorExtensionProof<TransactionAccumulatorHasher> {
        AccumulatorExtensionProof::<TransactionAccumulatorHasher>::new(
            frozen_subtree_roots,
            leaf_count,
            leaves
        )
    }
}

// This generates an arbitrary Block.
prop_compose! {
    pub fn arb_block(
    )(
        id in any::<HashValue>(),
        block_data in arb_block_data(),
        signature in arb_ed25519_signature(),
    ) -> Block {
        Block::new_for_testing(id, block_data, signature)
    }
}

// This generates an arbitrary BlockData.
prop_compose! {
    pub fn arb_block_data(
    )(
        epoch in any::<u64>(),
        round in any::<u64>(),
        timestamp_usecs in any::<u64>(),
        quorum_cert in arb_quorum_cert(),
        block_type in arb_block_type(),
    ) -> BlockData {
        BlockData::new_for_testing(epoch, round, timestamp_usecs, quorum_cert, block_type)
    }
}

// This generates an arbitrary BlockType::Proposal enum instance.
prop_compose! {
    pub fn arb_block_type_proposal(
    )(
        author in any::<AccountAddress>(),
        payload in prop::collection::vec(any::<SignedTransaction>(), 0..MAX_PROPOSAL_TRANSACTIONS),
    ) -> BlockType {
        BlockType::Proposal{
            payload,
            author
        }
    }
}

// This generates an arbitrary SafetyRulesInput::ConstructAndSignVote enum instance.
prop_compose! {
    pub fn arb_safety_rules_input_construct_and_sign_vote(
    )(
        signature in arb_ed25519_signature(),
        vote_proposal in arb_vote_proposal(),
    ) -> SafetyRulesInput {
        let maybe_signed_voted_proposal = MaybeSignedVoteProposal {
            vote_proposal,
            signature
        };
        SafetyRulesInput::ConstructAndSignVote(Box::new(maybe_signed_voted_proposal))
    }
}

// This generates an arbitrary SafetyRulesInput::Initialize enum instance.
prop_compose! {
    pub fn arb_safety_rules_input_initialize(
    )(
        more in any::<bool>(),
        ledger_info_with_sigs in prop::collection::vec(
            any::<LedgerInfoWithSignatures>(),
            0..MAX_NUM_LEDGER_INFO_WITH_SIGS
        ),
    ) -> SafetyRulesInput {
        SafetyRulesInput::Initialize(Box::new(EpochChangeProof::new(
            ledger_info_with_sigs,
            more,
        )))
    }
}

// This generates an arbitrary SafetyRulesInput::SignProposal enum instance.
prop_compose! {
    pub fn arb_safety_rules_input_sign_proposal(
    )(
        block_data in arb_block_data(),
    ) -> SafetyRulesInput {
        SafetyRulesInput::SignProposal(Box::new(block_data))
    }
}

// This generates an arbitrary SafetyRulesInput::SignTimeout enum instance.
prop_compose! {
    pub fn arb_safety_rules_input_sign_timeout(
    )(
        epoch in any::<u64>(),
        round in any::<u64>(),
    ) -> SafetyRulesInput {
        SafetyRulesInput::SignTimeout(Box::new(Timeout::new(epoch, round)))
    }
}

// This generates an arbitrary and optional Ed25519Signature.
prop_compose! {
    pub fn arb_ed25519_signature(
    )(
        include_signature in any::<bool>(),
        signature_bytes in prop::collection::vec(any::<u8>(), 0..Ed25519Signature::LENGTH),
    ) -> Option<Ed25519Signature> {
        if include_signature {
            if let Ok(signature) = Ed25519Signature::try_from(signature_bytes.as_slice()) {
                Some(signature)
            } else {
                None
            }
        } else {
            None
        }
    }
}

// This generates an arbitrary and optional EpochState.
prop_compose! {
    pub fn arb_epoch_state(
    )(
        include_epoch_state in any::<bool>(),
        epoch in any::<u64>(),
        address_to_validator_info in prop::collection::btree_map(
            any::<AccountAddress>(),
            arb_validator_consensus_info(),
            0..MAX_NUM_ADDR_TO_VALIDATOR_INFO
        ),
        quorum_voting_power in any::<u64>(),
        total_voting_power in any::<u64>(),
    ) -> Option<EpochState> {
        let verifier = ValidatorVerifier::new_for_testing(
            address_to_validator_info,
            quorum_voting_power,
            total_voting_power
        );
        if include_epoch_state {
            Some(EpochState {
                epoch,
                verifier
            })
        } else {
            None
        }
    }
}

// This generates an arbitrary QuorumCert.
prop_compose! {
    pub fn arb_quorum_cert(
    )(
        proposed_block_info_gen in any::<BlockInfoGen>(),
        parent_block_info_gen in any::<BlockInfoGen>(),
        mut proposed_account_info_universe in
            any_with::<AccountInfoUniverse>(NUM_UNIVERSE_ACCOUNTS),
        mut parent_account_info_universe in any_with::<AccountInfoUniverse>(NUM_UNIVERSE_ACCOUNTS),
        proposed_block_size in 1..MAX_BLOCK_SIZE,
        parent_block_size in 1..MAX_BLOCK_SIZE,
        signed_ledger_info in any::<LedgerInfoWithSignatures>(),
    ) -> QuorumCert {
        let proposed_block_info = proposed_block_info_gen.materialize(
            &mut proposed_account_info_universe,
            proposed_block_size
        );
        let parent_block_info = parent_block_info_gen.materialize(
            &mut parent_account_info_universe,
            parent_block_size
        );
        let vote_data = VoteData::new(proposed_block_info, parent_block_info);
        QuorumCert::new(vote_data, signed_ledger_info)
    }
}

// This generates an arbitrary ValidatorConsensusInfo.
prop_compose! {
    pub fn arb_validator_consensus_info(
    )(
        public_key in any::<Ed25519PublicKey>(),
        voting_power in any::<u64>(),
    ) -> ValidatorConsensusInfo {
        ValidatorConsensusInfo::new(public_key, voting_power)
    }
}

// This generates an arbitrary VoteProposal.
prop_compose! {
    pub fn arb_vote_proposal(
    )(
        accumulator_extension_proof in arb_accumulator_extension_proof(),
        block in arb_block(),
        next_epoch_state in arb_epoch_state()
    ) -> VoteProposal {
        VoteProposal::new(accumulator_extension_proof, block, next_epoch_state)
    }
}

// This generates an arbitrary BlockType enum.
fn arb_block_type() -> impl Strategy<Value = BlockType> {
    prop_oneof![
        arb_block_type_proposal(),
        Just(BlockType::NilBlock),
        Just(BlockType::Genesis),
    ]
}

// This generates an arbitrary SafetyRulesInput enum.
pub fn arb_safety_rules_input() -> impl Strategy<Value = SafetyRulesInput> {
    prop_oneof![
        Just(SafetyRulesInput::ConsensusState),
        arb_safety_rules_input_initialize(),
        arb_safety_rules_input_construct_and_sign_vote(),
        arb_safety_rules_input_sign_proposal(),
        arb_safety_rules_input_sign_timeout(),
    ]
}

// Note: these tests ensure that the various fuzzers are maintained (i.e., not broken
// at some time in the future and only discovered when a fuzz test fails).
#[cfg(test)]
mod tests {
    use crate::{fuzz_handle_message, fuzzing_utils::arb_safety_rules_input};
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(10))]

        #[test]
        fn handle_message_proptest(safety_rules_input in arb_safety_rules_input()) {
            let _ = fuzz_handle_message(safety_rules_input);
        }
    }
}
