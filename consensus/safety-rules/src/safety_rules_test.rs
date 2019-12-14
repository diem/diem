// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{test_utils, Error, InMemoryStorage, SafetyRules, TSafetyRules};
use consensus_types::{
    block::block_test_utils, block::Block, common::Round, quorum_cert::QuorumCert,
    timeout::Timeout, vote_proposal::VoteProposal,
};
use libra_crypto::hash::{CryptoHash, HashValue};
use libra_types::crypto_proxies::ValidatorSigner;
use std::sync::Arc;

type Proof = test_utils::Proof;

fn make_proposal_with_qc_and_proof(
    round: Round,
    proof: Proof,
    qc: QuorumCert,
    validator_signer: &ValidatorSigner,
) -> VoteProposal<Round> {
    test_utils::make_proposal_with_qc_and_proof(round, round, proof, qc, validator_signer)
}

fn make_proposal_with_parent(
    round: Round,
    parent: &VoteProposal<Round>,
    committed: Option<&VoteProposal<Round>>,
    validator_signer: &ValidatorSigner,
) -> VoteProposal<Round> {
    test_utils::make_proposal_with_parent(round, round, parent, committed, validator_signer)
}

#[test]
fn test_initial_state() {
    // Start from scratch, verify the state
    let block = Block::<Round>::make_genesis_block();

    let safety_rules = SafetyRules::<Round>::new(
        InMemoryStorage::default_storage(),
        Arc::new(ValidatorSigner::from_int(0)),
    );
    let state = safety_rules.consensus_state();
    assert_eq!(state.last_voted_round(), block.round());
    assert_eq!(state.preferred_round(), block.round());
}

#[test]
fn test_preferred_block_rule() {
    // Preferred block is the highest 2-chain head.
    let validator_signer = ValidatorSigner::from_int(0);
    let mut safety_rules = SafetyRules::<Round>::new(
        InMemoryStorage::default_storage(),
        Arc::new(validator_signer.clone()),
    );

    // build a tree of the following form:
    //             _____    _____
    //            /     \  /     \
    // genesis---a1  b1  b2  a2  b3  a3---a4
    //         \_____/ \_____/ \_____/
    //
    // PB should change from genesis to b1 and then a2.
    let genesis_block = Block::<Round>::make_genesis_block();
    let genesis_qc = block_test_utils::certificate_for_genesis();
    let round = genesis_block.round();

    let a1 = test_utils::make_proposal_with_qc(round + 1, genesis_qc.clone(), &validator_signer);
    let b1 = test_utils::make_proposal_with_qc(round + 2, genesis_qc.clone(), &validator_signer);
    let b2 = make_proposal_with_parent(round + 3, &a1, None, &validator_signer);
    let a2 = make_proposal_with_parent(round + 4, &b1, None, &validator_signer);
    let b3 = make_proposal_with_parent(round + 5, &b2, None, &validator_signer);
    let a3 = make_proposal_with_parent(round + 6, &a2, None, &validator_signer);
    let a4 = make_proposal_with_parent(round + 7, &a3, None, &validator_signer);

    safety_rules.update(a1.block().quorum_cert()).unwrap();
    assert_eq!(
        safety_rules.consensus_state().preferred_round(),
        genesis_block.round()
    );

    safety_rules.update(b1.block().quorum_cert()).unwrap();
    assert_eq!(
        safety_rules.consensus_state().preferred_round(),
        genesis_block.round()
    );

    safety_rules.update(a2.block().quorum_cert()).unwrap();
    assert_eq!(
        safety_rules.consensus_state().preferred_round(),
        genesis_block.round()
    );

    safety_rules.update(b2.block().quorum_cert()).unwrap();
    assert_eq!(
        safety_rules.consensus_state().preferred_round(),
        genesis_block.round()
    );

    safety_rules.update(a3.block().quorum_cert()).unwrap();
    assert_eq!(
        safety_rules.consensus_state().preferred_round(),
        b1.block().round()
    );

    safety_rules.update(b3.block().quorum_cert()).unwrap();
    assert_eq!(
        safety_rules.consensus_state().preferred_round(),
        b1.block().round()
    );

    safety_rules.update(a4.block().quorum_cert()).unwrap();
    assert_eq!(
        safety_rules.consensus_state().preferred_round(),
        a2.block().round()
    );
}

#[test]
/// Test the potential ledger info that we're going to use in case of voting
fn test_voting_potential_commit_id() {
    let validator_signer = ValidatorSigner::from_int(0);
    let mut safety_rules = SafetyRules::<Round>::new(
        InMemoryStorage::default_storage(),
        Arc::new(validator_signer.clone()),
    );

    // build a tree of the following form:
    //            _____
    //           /     \
    // genesis--a1  b1  a2--a3--a4--a5
    //        \_____/
    //
    // All the votes before a4 cannot produce any potential commits.
    // A potential commit for proposal a4 is a2, a potential commit for proposal a5 is a3.
    let genesis_block = Block::<Round>::make_genesis_block();
    let genesis_qc = block_test_utils::certificate_for_genesis();
    let round = genesis_block.round();

    let a1 = test_utils::make_proposal_with_qc(round + 1, genesis_qc.clone(), &validator_signer);
    let b1 = test_utils::make_proposal_with_qc(round + 2, genesis_qc.clone(), &validator_signer);
    let a2 = make_proposal_with_parent(round + 3, &a1, None, &validator_signer);
    let a3 = make_proposal_with_parent(round + 4, &a2, None, &validator_signer);
    let a4 = make_proposal_with_parent(round + 5, &a3, Some(&a2), &validator_signer);
    let a5 = make_proposal_with_parent(round + 6, &a4, Some(&a3), &validator_signer);

    for b in &[&a1, &b1, &a2, &a3] {
        safety_rules.update(b.block().quorum_cert()).unwrap();
        let vote = safety_rules.construct_and_sign_vote(b).unwrap();
        assert_eq!(vote.ledger_info().consensus_block_id(), HashValue::zero());
    }

    safety_rules.update(a4.block().quorum_cert()).unwrap();
    assert_eq!(
        safety_rules
            .construct_and_sign_vote(&a4)
            .unwrap()
            .ledger_info()
            .consensus_block_id(),
        a2.block().id(),
    );

    safety_rules.update(a5.block().quorum_cert()).unwrap();
    assert_eq!(
        safety_rules
            .construct_and_sign_vote(&a5)
            .unwrap()
            .ledger_info()
            .consensus_block_id(),
        a3.block().id(),
    );
}

#[test]
fn test_voting() {
    let validator_signer = ValidatorSigner::from_int(0);
    let mut safety_rules = SafetyRules::<Round>::new(
        InMemoryStorage::default_storage(),
        Arc::new(validator_signer.clone()),
    );

    // build a tree of the following form:
    //             _____    __________
    //            /     \  /          \
    // genesis---a1  b1  b2  a2---a3  b3  a4  b4
    //         \_____/ \_____/     \______/   /
    //                    \__________________/
    //
    //
    // We'll introduce the votes in the following order:
    // a1 (ok), potential_commit is None
    // b1 (ok), potential commit is None
    // a2 (ok), potential_commit is None
    // b2 (old proposal)
    // a3 (ok), potential commit is None
    // b3 (ok), potential commit is None
    // a4 (ok), potential commit is None
    // a4 (old proposal)
    // b4 (round lower then round of pb. PB: a2, parent(b4)=b2)
    let genesis_block = Block::<Round>::make_genesis_block();
    let genesis_qc = block_test_utils::certificate_for_genesis();
    let round = genesis_block.round();

    let a1 = test_utils::make_proposal_with_qc(round + 1, genesis_qc.clone(), &validator_signer);
    let b1 = test_utils::make_proposal_with_qc(round + 2, genesis_qc.clone(), &validator_signer);
    let b2 = make_proposal_with_parent(round + 3, &a1, None, &validator_signer);
    let a2 = make_proposal_with_parent(round + 4, &b1, None, &validator_signer);
    let a3 = make_proposal_with_parent(round + 5, &a2, None, &validator_signer);
    let b3 = make_proposal_with_parent(round + 6, &b2, None, &validator_signer);
    let a4 = make_proposal_with_parent(round + 7, &a3, None, &validator_signer);
    let b4 = make_proposal_with_parent(round + 8, &b2, None, &validator_signer);

    safety_rules.update(a1.block().quorum_cert()).unwrap();
    let mut vote = safety_rules.construct_and_sign_vote(&a1).unwrap();
    assert_eq!(vote.ledger_info().consensus_block_id(), HashValue::zero());

    safety_rules.update(b1.block().quorum_cert()).unwrap();
    vote = safety_rules.construct_and_sign_vote(&b1).unwrap();
    assert_eq!(vote.ledger_info().consensus_block_id(), HashValue::zero());

    safety_rules.update(a2.block().quorum_cert()).unwrap();
    vote = safety_rules.construct_and_sign_vote(&a2).unwrap();
    assert_eq!(vote.ledger_info().consensus_block_id(), HashValue::zero());

    safety_rules.update(b2.block().quorum_cert()).unwrap();

    assert_eq!(
        safety_rules.construct_and_sign_vote(&b2),
        Err(Error::OldProposal {
            last_voted_round: 4,
            proposal_round: 3,
        })
    );

    safety_rules.update(a3.block().quorum_cert()).unwrap();
    vote = safety_rules.construct_and_sign_vote(&a3).unwrap();
    assert_eq!(vote.ledger_info().consensus_block_id(), HashValue::zero());

    safety_rules.update(b3.block().quorum_cert()).unwrap();
    vote = safety_rules.construct_and_sign_vote(&b3).unwrap();
    assert_eq!(vote.ledger_info().consensus_block_id(), HashValue::zero());

    safety_rules.update(a4.block().quorum_cert()).unwrap();
    vote = safety_rules.construct_and_sign_vote(&a4).unwrap();
    assert_eq!(vote.ledger_info().consensus_block_id(), HashValue::zero());

    assert_eq!(
        safety_rules.construct_and_sign_vote(&a4),
        Err(Error::OldProposal {
            last_voted_round: 7,
            proposal_round: 7,
        })
    );

    safety_rules.update(b4.block().quorum_cert()).unwrap();
    assert_eq!(
        safety_rules.construct_and_sign_vote(&b4),
        Err(Error::ProposalRoundLowerThenPreferredBlock { preferred_round: 4 })
    );
}

#[test]
fn test_commit_rule_consecutive_rounds() {
    let validator_signer = ValidatorSigner::from_int(0);
    let safety_rules = SafetyRules::<Round>::new(
        InMemoryStorage::default_storage(),
        Arc::new(validator_signer.clone()),
    );

    // build a tree of the following form:
    //             ___________
    //            /           \
    // genesis---a1  b1---b2   a2---a3---a4
    //         \_____/
    //
    // a1 cannot be committed after a3 gathers QC because a1 and a2 are not consecutive
    // a2 can be committed after a4 gathers QC
    let genesis_block = Block::<Round>::make_genesis_block();
    let genesis_qc = block_test_utils::certificate_for_genesis();
    let round = genesis_block.round();

    let a1 = test_utils::make_proposal_with_qc(round + 1, genesis_qc.clone(), &validator_signer);
    let b1 = test_utils::make_proposal_with_qc(round + 2, genesis_qc.clone(), &validator_signer);
    let b2 = make_proposal_with_parent(round + 3, &b1, None, &validator_signer);
    let a2 = make_proposal_with_parent(round + 4, &a1, None, &validator_signer);
    let a3 = make_proposal_with_parent(round + 5, &a2, None, &validator_signer);
    let a4 = make_proposal_with_parent(round + 6, &a3, None, &validator_signer);

    assert_eq!(
        safety_rules
            .construct_ledger_info(a1.block())
            .consensus_block_id(),
        HashValue::zero(),
    );
    assert_eq!(
        safety_rules
            .construct_ledger_info(b1.block())
            .consensus_block_id(),
        HashValue::zero(),
    );
    assert_eq!(
        safety_rules
            .construct_ledger_info(b2.block())
            .consensus_block_id(),
        HashValue::zero(),
    );
    assert_eq!(
        safety_rules
            .construct_ledger_info(a2.block())
            .consensus_block_id(),
        HashValue::zero(),
    );
    assert_eq!(
        safety_rules
            .construct_ledger_info(a3.block())
            .consensus_block_id(),
        HashValue::zero(),
    );
    assert_eq!(
        safety_rules
            .construct_ledger_info(a4.block())
            .consensus_block_id(),
        a2.block().id(),
    );
}

#[test]
fn test_bad_execution_output() {
    let validator_signer = Arc::new(ValidatorSigner::from_int(0));
    let mut safety_rules =
        SafetyRules::<Round>::new(InMemoryStorage::default_storage(), validator_signer.clone());

    // build a tree of the following form:
    //                 _____
    //                /     \
    // genesis---a1--a2--a3  evil_a3
    //
    // evil_a3 attempts to append to a1 but fails append only check
    // a3 works as it properly extends a2
    let genesis_block = Block::<Round>::make_genesis_block();
    let genesis_qc = block_test_utils::certificate_for_genesis();
    let round = genesis_block.round();

    let a1 = test_utils::make_proposal_with_qc(round + 1, genesis_qc.clone(), &validator_signer);
    let a2 = make_proposal_with_parent(round + 2, &a1, None, &validator_signer);
    let a3 = make_proposal_with_parent(round + 3, &a2, None, &validator_signer);

    let a1_output = a1
        .accumulator_extension_proof()
        .verify(
            a1.block()
                .quorum_cert()
                .certified_block()
                .executed_state_id(),
        )
        .unwrap();

    let evil_proof = Proof::new(
        a1_output.frozen_subtree_roots().clone(),
        a1_output.num_leaves(),
        vec![Timeout::new(0, *a3.block().payload().unwrap()).hash()],
    );

    let evil_a3 = make_proposal_with_qc_and_proof(
        round,
        evil_proof,
        a3.block().quorum_cert().clone(),
        &validator_signer,
    );

    let evil_a3_block = safety_rules.construct_and_sign_vote(&evil_a3);
    assert!(evil_a3_block.is_err());

    let a3_block = safety_rules.construct_and_sign_vote(&a3);
    assert!(a3_block.is_ok());
}
