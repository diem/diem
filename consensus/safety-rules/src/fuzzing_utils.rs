// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::serializer::SafetyRulesInput;
#[cfg(any(test, feature = "fuzzing"))]
use consensus_types::block::Block;
use consensus_types::{
    block_data::{BlockData, BlockType},
    quorum_cert::QuorumCert,
    timeout::Timeout,
    vote_data::VoteData,
    vote_proposal::{MaybeSignedVoteProposal, VoteProposal},
};
use diem_crypto::{
    ed25519::{Ed25519PrivateKey, Ed25519PublicKey},
    hash::{HashValue, TransactionAccumulatorHasher},
    test_utils::TEST_SEED,
    traits::{SigningKey, Uniform},
};
use diem_types::{
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
use rand::{rngs::StdRng, SeedableRng};

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
        include_signature in any::<bool>(),
    ) -> Block {
        let signature = if include_signature {
            let mut rng = StdRng::from_seed(TEST_SEED);
            let private_key = Ed25519PrivateKey::generate(&mut rng);
            let signature = private_key.sign(&block_data);
            Some(signature)
        } else {
            None
        };
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

// This generates an arbitrary MaybeSignedVoteProposal.
prop_compose! {
    pub fn arb_maybe_signed_vote_proposal(
    )(
        accumulator_extension_proof in arb_accumulator_extension_proof(),
        block in arb_block(),
        next_epoch_state in arb_epoch_state(),
        include_signature in any::<bool>(),
    ) -> MaybeSignedVoteProposal {
        let vote_proposal = VoteProposal::new(accumulator_extension_proof, block, next_epoch_state);
        let signature = if include_signature {
            let mut rng = StdRng::from_seed(TEST_SEED);
            let private_key = Ed25519PrivateKey::generate(&mut rng);
            let signature = private_key.sign(&vote_proposal);
            Some(signature)
        } else {
            None
        };

        MaybeSignedVoteProposal {
            vote_proposal,
            signature
        }
    }
}

// This generates an arbitrary EpochChangeProof.
prop_compose! {
    pub fn arb_epoch_change_proof(
    )(
        more in any::<bool>(),
        ledger_info_with_sigs in prop::collection::vec(
            any::<LedgerInfoWithSignatures>(),
            0..MAX_NUM_LEDGER_INFO_WITH_SIGS
        ),
    ) -> EpochChangeProof {
        EpochChangeProof::new(
            ledger_info_with_sigs,
            more,
        )
    }
}

// This generates an arbitrary Timeout.
prop_compose! {
    pub fn arb_timeout(
    )(
        epoch in any::<u64>(),
        round in any::<u64>(),
    ) -> Timeout {
        Timeout::new(epoch, round)
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
        arb_epoch_change_proof().prop_map(|input| SafetyRulesInput::Initialize(Box::new(input))),
        arb_maybe_signed_vote_proposal()
            .prop_map(|input| { SafetyRulesInput::ConstructAndSignVote(Box::new(input)) }),
        arb_block_data().prop_map(|input| { SafetyRulesInput::SignProposal(Box::new(input)) }),
        arb_timeout().prop_map(|input| { SafetyRulesInput::SignTimeout(Box::new(input)) }),
    ]
}

#[cfg(any(test, feature = "fuzzing"))]
pub mod fuzzing {
    use crate::{error::Error, serializer::SafetyRulesInput, test_utils, TSafetyRules};
    use consensus_types::{
        block_data::BlockData, timeout::Timeout, vote::Vote, vote_proposal::MaybeSignedVoteProposal,
    };
    use diem_crypto::ed25519::Ed25519Signature;
    use diem_types::epoch_change::EpochChangeProof;

    pub fn fuzz_initialize(proof: EpochChangeProof) -> Result<(), Error> {
        let mut safety_rules = test_utils::test_safety_rules_uninitialized();
        safety_rules.initialize(&proof)
    }

    pub fn fuzz_construct_and_sign_vote(
        maybe_signed_vote_proposal: MaybeSignedVoteProposal,
    ) -> Result<Vote, Error> {
        let mut safety_rules = test_utils::test_safety_rules();
        safety_rules.construct_and_sign_vote(&maybe_signed_vote_proposal)
    }

    pub fn fuzz_handle_message(safety_rules_input: SafetyRulesInput) -> Result<Vec<u8>, Error> {
        // Create a safety rules serializer test instance for fuzzing
        let mut serializer_service = test_utils::test_serializer();

        // encode the safety_rules_input and fuzz the handle_message() method
        if let Ok(safety_rules_input) = serde_json::to_vec(&safety_rules_input) {
            serializer_service.handle_message(safety_rules_input)
        } else {
            Err(Error::SerializationError(
                "Unable to serialize safety rules input for fuzzer!".into(),
            ))
        }
    }

    pub fn fuzz_sign_proposal(block_data: &BlockData) -> Result<Ed25519Signature, Error> {
        let mut safety_rules = test_utils::test_safety_rules();
        safety_rules.sign_proposal(block_data)
    }

    pub fn fuzz_sign_timeout(timeout: Timeout) -> Result<Ed25519Signature, Error> {
        let mut safety_rules = test_utils::test_safety_rules();
        safety_rules.sign_timeout(&timeout)
    }
}

// Note: these tests ensure that the various fuzzers are maintained (i.e., not broken
// at some time in the future and only discovered when a fuzz test fails).
#[cfg(test)]
mod tests {
    use crate::{
        fuzzing::{
            fuzz_construct_and_sign_vote, fuzz_handle_message, fuzz_initialize, fuzz_sign_proposal,
            fuzz_sign_timeout,
        },
        fuzzing_utils::{
            arb_block_data, arb_epoch_change_proof, arb_maybe_signed_vote_proposal,
            arb_safety_rules_input, arb_timeout,
        },
    };
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(10))]

        #[test]
        fn handle_message_proptest(input in arb_safety_rules_input()) {
            let _ = fuzz_handle_message(input);
        }

        #[test]
        fn initialize_proptest(input in arb_epoch_change_proof()) {
            let _ = fuzz_initialize(input);
        }

        #[test]
        fn construct_and_sign_vote_proptest(input in arb_maybe_signed_vote_proposal()) {
            let _ = fuzz_construct_and_sign_vote(input);
        }

        #[test]
        fn sign_proposal_proptest(input in arb_block_data()) {
            let _ = fuzz_sign_proposal(&input);
        }

        #[test]
        fn sign_timeout_proptest(input in arb_timeout()) {
            let _ = fuzz_sign_timeout(input);
        }
    }
}
