// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{test_utils, Error, SafetyRules, TSafetyRules};
use consensus_types::{
    block::{block_test_utils::random_payload, Block},
    common::Round,
    quorum_cert::QuorumCert,
    timeout::Timeout,
    vote_proposal::VoteProposal,
};
use libra_crypto::hash::{CryptoHash, HashValue};
use libra_types::{
    epoch_change::EpochChangeProof,
    epoch_state::EpochState,
    ledger_info::{LedgerInfo, LedgerInfoWithSignatures},
    on_chain_config::ValidatorSet,
    validator_info::ValidatorInfo,
    validator_signer::ValidatorSigner,
    validator_verifier::ValidatorVerifier,
};
use std::collections::BTreeMap;

type Proof = test_utils::Proof;

fn make_genesis(signer: &ValidatorSigner) -> (EpochChangeProof, QuorumCert) {
    let validator_info =
        ValidatorInfo::new_with_test_network_keys(signer.author(), signer.public_key(), 1);
    let validator_set = ValidatorSet::new(vec![validator_info]);
    let li = LedgerInfo::mock_genesis(Some(validator_set));
    let block = Block::make_genesis_block_from_ledger_info(&li);
    let qc = QuorumCert::certificate_for_genesis_from_ledger_info(&li, block.id());
    let lis = LedgerInfoWithSignatures::new(li, BTreeMap::new());
    let proof = EpochChangeProof::new(vec![lis], false);
    (proof, qc)
}

fn make_proposal_with_qc_and_proof(
    round: Round,
    proof: Proof,
    qc: QuorumCert,
    signer: &ValidatorSigner,
) -> VoteProposal {
    test_utils::make_proposal_with_qc_and_proof(vec![], round, proof, qc, signer)
}

fn make_proposal_with_parent(
    round: Round,
    parent: &VoteProposal,
    committed: Option<&VoteProposal>,
    signer: &ValidatorSigner,
) -> VoteProposal {
    test_utils::make_proposal_with_parent(vec![], round, parent, committed, signer)
}

type Callback = fn() -> (Box<dyn TSafetyRules>, ValidatorSigner);

pub fn run_test_suite(func: Callback) {
    test_bad_execution_output(func);
    test_commit_rule_consecutive_rounds(func);
    test_end_to_end(func);
    test_initialize(func);
    test_preferred_block_rule(func);
    test_sign_timeout(func);
    test_voting(func);
    test_voting_potential_commit_id(func);
    test_voting_bad_epoch(func);
    test_sign_old_proposal(func);
    test_sign_proposal_with_bad_signer(func);
    test_sign_proposal_with_invalid_qc(func);
    test_sign_proposal_with_early_preferred_round(func);
    test_uninitialized_signer(func);
    test_reconcile_key(func);
    test_a_missing_validator(func);
    // test_a_missing_validator_key(func);
}

fn test_bad_execution_output(func: Callback) {
    // build a tree of the following form:
    //                 _____
    //                /     \
    // genesis---a1--a2--a3  evil_a3
    //
    // evil_a3 attempts to append to a1 but fails append only check
    // a3 works as it properly extends a2
    let (mut safety_rules, signer) = func();

    let (proof, genesis_qc) = make_genesis(&signer);
    let round = genesis_qc.certified_block().round();

    let a1 = test_utils::make_proposal_with_qc(round + 1, genesis_qc, &signer);
    let a2 = make_proposal_with_parent(round + 2, &a1, None, &signer);
    let a3 = make_proposal_with_parent(round + 3, &a2, None, &signer);

    safety_rules.initialize(&proof).unwrap();
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
        vec![Timeout::new(0, a3.block().round()).hash()],
    );

    let evil_a3 = make_proposal_with_qc_and_proof(
        round,
        evil_proof,
        a3.block().quorum_cert().clone(),
        &signer,
    );

    let evil_a3_block = safety_rules.construct_and_sign_vote(&evil_a3);
    assert!(evil_a3_block.is_err());

    let a3_block = safety_rules.construct_and_sign_vote(&a3);
    assert!(a3_block.is_ok());
}

fn test_commit_rule_consecutive_rounds(func: Callback) {
    // build a tree of the following form:
    //             ___________
    //            /           \
    // genesis---a1  b1---b2   a2---a3---a4
    //         \_____/
    //
    // a1 cannot be committed after a3 gathers QC because a1 and a2 are not consecutive
    // a2 can be committed after a4 gathers QC
    let (mut safety_rules, signer) = func();

    let (proof, genesis_qc) = make_genesis(&signer);
    let round = genesis_qc.certified_block().round();

    let a1 = test_utils::make_proposal_with_qc(round + 1, genesis_qc.clone(), &signer);
    let b1 = test_utils::make_proposal_with_qc(round + 2, genesis_qc, &signer);
    let b2 = make_proposal_with_parent(round + 3, &b1, None, &signer);
    let a2 = make_proposal_with_parent(round + 4, &a1, None, &signer);
    let a3 = make_proposal_with_parent(round + 5, &a2, None, &signer);
    let a4 = make_proposal_with_parent(round + 6, &a3, Some(&a2), &signer);

    safety_rules.initialize(&proof).unwrap();
    safety_rules.construct_and_sign_vote(&a1).unwrap();
    safety_rules.construct_and_sign_vote(&b1).unwrap();
    safety_rules.construct_and_sign_vote(&b2).unwrap();
    safety_rules.construct_and_sign_vote(&a2).unwrap();
    safety_rules.construct_and_sign_vote(&a3).unwrap();
    safety_rules.construct_and_sign_vote(&a4).unwrap();
}

fn test_end_to_end(func: Callback) {
    let (mut safety_rules, signer) = func();

    let (proof, genesis_qc) = make_genesis(&signer);
    let round = genesis_qc.certified_block().round();

    let data = random_payload(2048);

    let p0 = test_utils::make_proposal_with_qc(round + 1, genesis_qc.clone(), &signer);
    let p1 = test_utils::make_proposal_with_parent(data.clone(), round + 2, &p0, None, &signer);
    let p2 = test_utils::make_proposal_with_parent(data.clone(), round + 3, &p1, None, &signer);
    let p3 = test_utils::make_proposal_with_parent(data, round + 4, &p2, Some(&p0), &signer);

    let state = safety_rules.consensus_state().unwrap();
    assert_eq!(
        state.last_voted_round(),
        genesis_qc.certified_block().round()
    );
    assert_eq!(
        state.preferred_round(),
        genesis_qc.certified_block().round()
    );

    safety_rules.initialize(&proof).unwrap();
    safety_rules.update(p0.block().quorum_cert()).unwrap();
    safety_rules.construct_and_sign_vote(&p0).unwrap();
    safety_rules.update(p1.block().quorum_cert()).unwrap();
    safety_rules.construct_and_sign_vote(&p1).unwrap();
    safety_rules.update(p2.block().quorum_cert()).unwrap();
    safety_rules.construct_and_sign_vote(&p2).unwrap();
    safety_rules.update(p3.block().quorum_cert()).unwrap();
    safety_rules.construct_and_sign_vote(&p3).unwrap();

    let state = safety_rules.consensus_state().unwrap();
    assert_eq!(state.last_voted_round(), round + 4);
    assert_eq!(state.preferred_round(), round + 2);
}

/// Initialize from scratch, ensure that SafetyRules can properly initialize from a Waypoint and
/// that it rejects invalid LedgerInfos or those that do not match.
fn test_initialize(func: Callback) {
    let (mut safety_rules, signer) = func();

    let state = safety_rules.consensus_state().unwrap();
    assert_eq!(state.last_voted_round(), 0);
    assert_eq!(state.preferred_round(), 0);
    assert_eq!(state.epoch(), 1);

    let (proof, _genesis_qc) = make_genesis(&signer);
    safety_rules.initialize(&proof).unwrap();

    let signer1 = ValidatorSigner::from_int(1);
    let (bad_proof, _bad_genesis_qc) = make_genesis(&signer1);

    match safety_rules.initialize(&bad_proof) {
        Err(Error::WaypointMismatch(_)) => (),
        _ => panic!("Unexpected output"),
    };
}

fn test_preferred_block_rule(func: Callback) {
    // Preferred block is the highest 2-chain head.
    //
    // build a tree of the following form:
    //             _____    _____
    //            /     \  /     \
    // genesis---a1  b1  b2  a2  b3  a3---a4
    //         \_____/ \_____/ \_____/
    //
    // PB should change from genesis to b1 and then a2.
    let (mut safety_rules, signer) = func();

    let (proof, genesis_qc) = make_genesis(&signer);
    let genesis_round = genesis_qc.certified_block().round();
    let round = genesis_round;

    let a1 = test_utils::make_proposal_with_qc(round + 1, genesis_qc.clone(), &signer);
    let b1 = test_utils::make_proposal_with_qc(round + 2, genesis_qc, &signer);
    let b2 = make_proposal_with_parent(round + 3, &a1, None, &signer);
    let a2 = make_proposal_with_parent(round + 4, &b1, None, &signer);
    let b3 = make_proposal_with_parent(round + 5, &b2, None, &signer);
    let a3 = make_proposal_with_parent(round + 6, &a2, None, &signer);
    let a4 = make_proposal_with_parent(round + 7, &a3, None, &signer);

    safety_rules.initialize(&proof).unwrap();

    safety_rules.update(a1.block().quorum_cert()).unwrap();
    assert_eq!(
        safety_rules.consensus_state().unwrap().preferred_round(),
        genesis_round
    );

    safety_rules.update(b1.block().quorum_cert()).unwrap();
    assert_eq!(
        safety_rules.consensus_state().unwrap().preferred_round(),
        genesis_round
    );

    safety_rules.update(a2.block().quorum_cert()).unwrap();
    assert_eq!(
        safety_rules.consensus_state().unwrap().preferred_round(),
        genesis_round
    );

    safety_rules.update(b2.block().quorum_cert()).unwrap();
    assert_eq!(
        safety_rules.consensus_state().unwrap().preferred_round(),
        genesis_round
    );

    safety_rules.update(a3.block().quorum_cert()).unwrap();
    assert_eq!(
        safety_rules.consensus_state().unwrap().preferred_round(),
        b1.block().round()
    );

    safety_rules.update(b3.block().quorum_cert()).unwrap_err();
    assert_eq!(
        safety_rules.consensus_state().unwrap().preferred_round(),
        b1.block().round()
    );

    safety_rules.update(a4.block().quorum_cert()).unwrap();
    assert_eq!(
        safety_rules.consensus_state().unwrap().preferred_round(),
        a2.block().round()
    );
}

/// Verify first that we can successfully sign a timeout on the correct conditions, then ensure
/// that poorly set last_voted_rounds both historical and in the future fail as well as
/// synchronization issues on preferred round are correct. Effectivelly ensure that equivocation is
/// impossible for signing timeouts.
fn test_sign_timeout(func: Callback) {
    let (mut safety_rules, signer) = func();

    let (proof, genesis_qc) = make_genesis(&signer);
    let round = genesis_qc.certified_block().round();
    let epoch = genesis_qc.certified_block().epoch();

    let p0 = test_utils::make_proposal_with_qc(round + 1, genesis_qc, &signer);
    let p1 = make_proposal_with_parent(round + 2, &p0, None, &signer);
    let p2 = make_proposal_with_parent(round + 3, &p1, None, &signer);
    let p3 = make_proposal_with_parent(round + 4, &p2, None, &signer);
    let p4 = make_proposal_with_parent(round + 5, &p3, None, &signer);

    safety_rules.initialize(&proof).unwrap();
    safety_rules.update(p0.block().quorum_cert()).unwrap();

    // Verify multiple signings are the same
    let timeout = Timeout::new(epoch, p0.block().round());
    let sign1 = safety_rules.sign_timeout(&timeout).unwrap();
    let sign2 = safety_rules.sign_timeout(&timeout).unwrap();
    assert_eq!(sign1, sign2);

    // Verify can sign last_voted_round + 1
    let timeout_plus_1 = Timeout::new(timeout.epoch(), timeout.round() + 1);
    safety_rules.sign_timeout(&timeout_plus_1).unwrap();

    // Verify cannot sign round older rounds now
    let actual_err = safety_rules.sign_timeout(&timeout).unwrap_err();
    let expected_err = Error::BadTimeoutLastVotedRound(timeout.round(), timeout.round() + 1);
    assert_eq!(actual_err, expected_err);

    // Verify cannot sign last_voted_round < vote < preferred_round
    safety_rules.update(p4.block().quorum_cert()).unwrap();
    let preferred_round = p4.block().quorum_cert().parent_block().round();
    let ptimeout = Timeout::new(timeout.epoch(), preferred_round - 1);
    let actual_err = safety_rules.sign_timeout(&ptimeout).unwrap_err();
    let expected_err = Error::BadTimeoutPreferredRound(ptimeout.round(), preferred_round);
    assert_eq!(actual_err, expected_err);

    // Verify cannot sign for different epoch
    let etimeout = Timeout::new(timeout.epoch() + 1, round + 1);
    let actual_err = safety_rules.sign_timeout(&etimeout).unwrap_err();
    let expected_err = Error::IncorrectEpoch(etimeout.epoch(), timeout.epoch());
    assert_eq!(actual_err, expected_err);
}

fn test_voting(func: Callback) {
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
    let (mut safety_rules, signer) = func();

    let (proof, genesis_qc) = make_genesis(&signer);
    let round = genesis_qc.certified_block().round();

    let a1 = test_utils::make_proposal_with_qc(round + 1, genesis_qc.clone(), &signer);
    let b1 = test_utils::make_proposal_with_qc(round + 2, genesis_qc, &signer);
    let b2 = make_proposal_with_parent(round + 3, &a1, None, &signer);
    let a2 = make_proposal_with_parent(round + 4, &b1, None, &signer);
    let a3 = make_proposal_with_parent(round + 5, &a2, None, &signer);
    let b3 = make_proposal_with_parent(round + 6, &b2, None, &signer);
    let a4 = make_proposal_with_parent(round + 7, &a3, None, &signer);
    let b4 = make_proposal_with_parent(round + 8, &b2, None, &signer);

    safety_rules.initialize(&proof).unwrap();

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

    safety_rules.update(b3.block().quorum_cert()).unwrap_err();
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

    safety_rules.update(b4.block().quorum_cert()).unwrap_err();
    assert_eq!(
        safety_rules.construct_and_sign_vote(&b4),
        Err(Error::ProposalRoundLowerThenPreferredBlock { preferred_round: 4 })
    );
}

fn test_voting_bad_epoch(func: Callback) {
    // Test to verify epoch is the same between parent and proposed in a vote proposal
    // genesis--a1 -> a2 fails due to jumping to a different epoch
    let (mut safety_rules, signer) = func();

    let (proof, genesis_qc) = make_genesis(&signer);
    let round = genesis_qc.certified_block().round();

    let a1 = test_utils::make_proposal_with_qc(round + 1, genesis_qc, &signer);
    let a2 = test_utils::make_proposal_with_parent_and_overrides(
        vec![],
        round + 3,
        &a1,
        None,
        &signer,
        Some(21),
        None,
    );
    safety_rules.initialize(&proof).unwrap();
    safety_rules.update(a1.block().quorum_cert()).unwrap();

    assert_eq!(
        safety_rules.construct_and_sign_vote(&a2),
        Err(Error::IncorrectEpoch(21, 1))
    );
}

fn test_voting_potential_commit_id(func: Callback) {
    // Test the potential ledger info that we're going to use in case of voting
    // build a tree of the following form:
    //            _____
    //           /     \
    // genesis--a1  b1  a2--a3--a4--a5
    //        \_____/
    //
    // All the votes before a4 cannot produce any potential commits.
    // A potential commit for proposal a4 is a2, a potential commit for proposal a5 is a3.
    let (mut safety_rules, signer) = func();

    let (proof, genesis_qc) = make_genesis(&signer);
    let round = genesis_qc.certified_block().round();

    let a1 = test_utils::make_proposal_with_qc(round + 1, genesis_qc.clone(), &signer);
    let b1 = test_utils::make_proposal_with_qc(round + 2, genesis_qc, &signer);
    let a2 = make_proposal_with_parent(round + 3, &a1, None, &signer);
    let a3 = make_proposal_with_parent(round + 4, &a2, None, &signer);
    let a4 = make_proposal_with_parent(round + 5, &a3, Some(&a2), &signer);
    let a5 = make_proposal_with_parent(round + 6, &a4, Some(&a3), &signer);

    safety_rules.initialize(&proof).unwrap();

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

fn test_sign_old_proposal(func: Callback) {
    // Test to sign a proposal which makes no progress, compared with last voted round

    let (mut safety_rules, signer) = func();

    let (proof, genesis_qc) = make_genesis(&signer);
    let round = genesis_qc.certified_block().round();
    safety_rules.initialize(&proof).unwrap();

    let a1 = test_utils::make_proposal_with_qc(round, genesis_qc, &signer);
    safety_rules.update(a1.block().quorum_cert()).unwrap();
    let err = safety_rules
        .sign_proposal(a1.block().block_data().clone())
        .unwrap_err();
    assert_eq!(
        err,
        Error::OldProposal {
            last_voted_round: 0,
            proposal_round: 0,
        }
    );
}

fn test_sign_proposal_with_bad_signer(func: Callback) {
    // Test to sign a proposal signed by an unrecognizable signer

    let (mut safety_rules, signer) = func();

    let (proof, genesis_qc) = make_genesis(&signer);
    let round = genesis_qc.certified_block().round();
    safety_rules.initialize(&proof).unwrap();

    let a1 = test_utils::make_proposal_with_qc(round + 1, genesis_qc, &signer);
    safety_rules.update(a1.block().quorum_cert()).unwrap();
    safety_rules
        .sign_proposal(a1.block().block_data().clone())
        .unwrap();

    let bad_signer = ValidatorSigner::from_int(0xef);
    let a2 = make_proposal_with_parent(round + 2, &a1, None, &bad_signer);
    let err = safety_rules
        .sign_proposal(a2.block().block_data().clone())
        .unwrap_err();
    assert_eq!(
        err,
        Error::InvalidProposal("Proposal author is not validator signer!".into())
    );
}

fn test_sign_proposal_with_invalid_qc(func: Callback) {
    // Test to sign a proposal with an invalid qc inherited from proposal a2, which
    // is signed by a bad_signer.

    let (mut safety_rules, signer) = func();

    let (proof, genesis_qc) = make_genesis(&signer);
    let round = genesis_qc.certified_block().round();
    safety_rules.initialize(&proof).unwrap();

    let a1 = test_utils::make_proposal_with_qc(round + 1, genesis_qc, &signer);
    safety_rules.update(a1.block().quorum_cert()).unwrap();
    safety_rules
        .sign_proposal(a1.block().block_data().clone())
        .unwrap();

    let bad_signer = ValidatorSigner::from_int(0xef);
    let a2 = make_proposal_with_parent(round + 2, &a1, Some(&a1), &bad_signer);
    let a3 =
        test_utils::make_proposal_with_qc(round + 3, a2.block().quorum_cert().clone(), &signer);
    let err = safety_rules
        .sign_proposal(a3.block().block_data().clone())
        .unwrap_err();
    assert_eq!(
        err,
        Error::InvalidQuorumCertificate("Fail to verify QuorumCert".into())
    );
}

fn test_sign_proposal_with_early_preferred_round(func: Callback) {
    let (mut safety_rules, signer) = func();

    let (proof, genesis_qc) = make_genesis(&signer);
    let round = genesis_qc.certified_block().round();
    safety_rules.initialize(&proof).unwrap();

    let a1 = test_utils::make_proposal_with_qc(round + 1, genesis_qc, &signer);
    safety_rules.update(a1.block().quorum_cert()).unwrap();
    safety_rules
        .sign_proposal(a1.block().block_data().clone())
        .unwrap();

    // Update preferred round with a few legal proposals
    let a2 = make_proposal_with_parent(round + 2, &a1, None, &signer);
    let a3 = make_proposal_with_parent(round + 3, &a2, None, &signer);
    let a4 = make_proposal_with_parent(round + 4, &a3, Some(&a2), &signer);
    safety_rules.update(a2.block().quorum_cert()).unwrap();
    safety_rules.update(a3.block().quorum_cert()).unwrap();
    safety_rules.update(a4.block().quorum_cert()).unwrap();

    let a5 = make_proposal_with_qc_and_proof(
        round + 5,
        test_utils::empty_proof(),
        a1.block().quorum_cert().clone(),
        &signer,
    );
    let err = safety_rules
        .sign_proposal(a5.block().block_data().clone())
        .unwrap_err();
    assert_eq!(
        err,
        Error::InvalidQuorumCertificate("Preferred round too early".into())
    );
}

fn test_uninitialized_signer(func: Callback) {
    // Testing for an uninitialized Option<ValidatorSigner>

    let (mut safety_rules, signer) = func();

    let (proof, genesis_qc) = make_genesis(&signer);
    let round = genesis_qc.certified_block().round();

    let a1 = test_utils::make_proposal_with_qc(round + 1, genesis_qc, &signer);
    safety_rules.update(a1.block().quorum_cert()).unwrap_err();

    safety_rules.initialize(&proof).unwrap();
    safety_rules.update(a1.block().quorum_cert()).unwrap();
}

// Tests for fetching a missing validator key from persistent storage.
// Ideally under this circumstance, safety rules should panic. However,
// at current stage of implementation the upper layer (i.e. RoundManager) is
// unable to handle such failure (actually it is unable ANY failure
// during key reconciliation!). For more fail cases, refer to debug messages in
// reconcile_key()
#[allow(dead_code)]
fn test_a_missing_validator_key(func: Callback) {
    let (mut safety_rules, signer) = func();
    let (proof, genesis_qc) = make_genesis(&signer);
    let round = genesis_qc.certified_block().round();

    safety_rules.initialize(&proof).unwrap();

    let a1 = test_utils::make_proposal_with_qc(round + 1, genesis_qc, &signer);

    let mut next_epoch_state = EpochState::empty();
    next_epoch_state.epoch = 1;
    let rand_signer = ValidatorSigner::random([0xfu8; 32]);
    next_epoch_state.verifier =
        ValidatorVerifier::new_single(signer.author(), rand_signer.public_key());
    let a2 = test_utils::make_proposal_with_parent_and_overrides(
        vec![],
        round + 2,
        &a1,
        Some(&a1),
        &signer,
        Some(1),
        Some(next_epoch_state),
    );
    safety_rules.update(a2.block().quorum_cert()).unwrap_err();
}

fn test_a_missing_validator(func: Callback) {
    // Testing for an validator missing from the validator set
    // It does so by updating the safey rule to an epoch state, which does not contain the
    // current validator. The validator later fails to verify QC since it is no longer in the
    // validator set.

    let (mut safety_rules, signer) = func();

    let (proof, genesis_qc) = make_genesis(&signer);
    let round = genesis_qc.certified_block().round();

    safety_rules.initialize(&proof).unwrap();

    let a1 = test_utils::make_proposal_with_qc(round + 1, genesis_qc, &signer);

    let mut next_epoch_state = EpochState::empty();
    next_epoch_state.epoch = 1;
    let rand_signer = ValidatorSigner::random([0xfu8; 32]);
    next_epoch_state.verifier =
        ValidatorVerifier::new_single(rand_signer.author(), rand_signer.public_key());
    let a2 = test_utils::make_proposal_with_parent_and_overrides(
        vec![],
        round + 2,
        &a1,
        Some(&a1),
        &signer,
        Some(1),
        Some(next_epoch_state),
    );
    safety_rules.update(a2.block().quorum_cert()).unwrap();

    let a3 = test_utils::make_proposal_with_parent_and_overrides(
        vec![],
        round + 3,
        &a2,
        Some(&a2),
        &signer,
        Some(1),
        None,
    );
    let err = safety_rules.update(a3.block().quorum_cert()).unwrap_err();
    assert_eq!(
        err,
        Error::InvalidQuorumCertificate("Fail to verify QuorumCert".into())
    );
}

fn test_reconcile_key(_func: Callback) {
    // Test to verify desired consensus key can be retrieved according to validator set.
    // It does so by updating the safey rule to a desired epoch state, reconciling old signer key
    // with the new one. Later when it tries to verify the QC signed by the old signer key, safety
    // rules fails the check.

    // Initialize the storage with two versions of signer keys
    let signer = ValidatorSigner::from_int(0);
    let mut storage = test_utils::test_storage(&signer);
    let new_pub_key = test_utils::rotate_consensus_key(&mut storage).unwrap();
    let mut safety_rules = Box::new(SafetyRules::new(signer.author(), storage));

    let (proof, genesis_qc) = make_genesis(&signer);
    let round = genesis_qc.certified_block().round();

    safety_rules.initialize(&proof).unwrap();

    let a1 = test_utils::make_proposal_with_qc(round + 1, genesis_qc, &signer);
    safety_rules.update(a1.block().quorum_cert()).unwrap();

    // Update validator epoch state, reconciling the old key with the new pub key
    let mut next_epoch_state = EpochState::empty();
    next_epoch_state.epoch = 1;
    next_epoch_state.verifier = ValidatorVerifier::new_single(signer.author(), new_pub_key);
    let a2 = test_utils::make_proposal_with_parent_and_overrides(
        vec![],
        round + 2,
        &a1,
        Some(&a1),
        &signer,
        Some(1),
        Some(next_epoch_state),
    );
    safety_rules.update(a2.block().quorum_cert()).unwrap();

    // Verification fails for proposal signed by the outdated key
    let outdated_signer = &signer;
    let a3 = test_utils::make_proposal_with_parent_and_overrides(
        vec![],
        round + 3,
        &a2,
        Some(&a2),
        outdated_signer,
        Some(1),
        None,
    );
    let err = safety_rules.update(a3.block().quorum_cert()).unwrap_err();
    assert_eq!(
        err,
        Error::InvalidQuorumCertificate("Fail to verify QuorumCert".into())
    );
}
