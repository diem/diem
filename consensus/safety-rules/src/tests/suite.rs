// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{test_utils, Error, TSafetyRules};
use consensus_types::{
    block::Block,
    common::{Payload, Round},
    quorum_cert::QuorumCert,
    timeout::Timeout,
    vote_proposal::VoteProposal,
};
use libra_crypto::hash::{CryptoHash, HashValue};
use libra_types::{
    epoch_change::EpochChangeProof,
    ledger_info::{LedgerInfo, LedgerInfoWithSignatures},
    on_chain_config::ValidatorSet,
    validator_info::ValidatorInfo,
    validator_signer::ValidatorSigner,
};
use rand::Rng;
use std::collections::BTreeMap;

type Proof = test_utils::Proof;

fn make_genesis<T: Payload>(signer: &ValidatorSigner) -> (EpochChangeProof, QuorumCert) {
    let validator_info =
        ValidatorInfo::new_with_test_network_keys(signer.author(), signer.public_key(), 1);
    let validator_set = ValidatorSet::new(vec![validator_info]);
    let li = LedgerInfo::mock_genesis(Some(validator_set));
    let block = Block::<T>::make_genesis_block_from_ledger_info(&li);
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
) -> VoteProposal<Round> {
    test_utils::make_proposal_with_qc_and_proof(round, round, proof, qc, signer)
}

fn make_proposal_with_parent(
    round: Round,
    parent: &VoteProposal<Round>,
    committed: Option<&VoteProposal<Round>>,
    signer: &ValidatorSigner,
) -> VoteProposal<Round> {
    test_utils::make_proposal_with_parent(round, round, parent, committed, signer)
}

type RoundCallback = fn() -> (Box<dyn TSafetyRules<Round>>, ValidatorSigner);
type ByteArrayCallback = fn() -> (Box<dyn TSafetyRules<Vec<u8>>>, ValidatorSigner);

pub fn run_test_suite(round_func: RoundCallback, byte_func: ByteArrayCallback) {
    test_bad_execution_output(round_func);
    test_commit_rule_consecutive_rounds(round_func);
    test_end_to_end(byte_func);
    test_initialize(round_func);
    test_preferred_block_rule(round_func);
    test_sign_timeout(round_func);
    test_voting(round_func);
    test_voting_potential_commit_id(round_func);
    test_voting_bad_epoch(round_func);
}

fn test_bad_execution_output(func: RoundCallback) {
    // build a tree of the following form:
    //                 _____
    //                /     \
    // genesis---a1--a2--a3  evil_a3
    //
    // evil_a3 attempts to append to a1 but fails append only check
    // a3 works as it properly extends a2
    let (mut safety_rules, signer) = func();

    let (proof, genesis_qc) = make_genesis::<Round>(&signer);
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
        vec![Timeout::new(0, *a3.block().payload().unwrap()).hash()],
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

fn test_commit_rule_consecutive_rounds(func: RoundCallback) {
    // build a tree of the following form:
    //             ___________
    //            /           \
    // genesis---a1  b1---b2   a2---a3---a4
    //         \_____/
    //
    // a1 cannot be committed after a3 gathers QC because a1 and a2 are not consecutive
    // a2 can be committed after a4 gathers QC
    let (mut safety_rules, signer) = func();

    let (proof, genesis_qc) = make_genesis::<Round>(&signer);
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

fn test_end_to_end(func: ByteArrayCallback) {
    let (mut safety_rules, signer) = func();

    let (proof, genesis_qc) = make_genesis::<Round>(&signer);
    let round = genesis_qc.certified_block().round();

    let mut rng = rand::thread_rng();
    let data: Vec<u8> = (0..2048).map(|_| rng.gen::<u8>()).collect();

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
fn test_initialize(func: RoundCallback) {
    let (mut safety_rules, signer) = func();

    let state = safety_rules.consensus_state().unwrap();
    assert_eq!(state.last_voted_round(), 0);
    assert_eq!(state.preferred_round(), 0);
    assert_eq!(state.epoch(), 1);

    let (proof, _genesis_qc) = make_genesis::<Round>(&signer);
    safety_rules.initialize(&proof).unwrap();

    let signer1 = ValidatorSigner::from_int(1);
    let (bad_proof, _bad_genesis_qc) = make_genesis::<Round>(&signer1);

    match safety_rules.initialize(&bad_proof) {
        Err(Error::WaypointMismatch(_)) => (),
        _ => panic!("Unexpected output"),
    };
}

fn test_preferred_block_rule(func: RoundCallback) {
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

    let (proof, genesis_qc) = make_genesis::<Round>(&signer);
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
fn test_sign_timeout(func: RoundCallback) {
    let (mut safety_rules, signer) = func();

    let (proof, genesis_qc) = make_genesis::<Round>(&signer);
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

fn test_voting(func: RoundCallback) {
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

    let (proof, genesis_qc) = make_genesis::<Round>(&signer);
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

fn test_voting_bad_epoch(func: RoundCallback) {
    // Test to verify epoch is the same between parent and proposed in a vote proposal
    // genesis--a1 -> a2 fails due to jumping to a different epoch
    let (mut safety_rules, signer) = func();

    let (proof, genesis_qc) = make_genesis::<Round>(&signer);
    let round = genesis_qc.certified_block().round();

    let a1 = test_utils::make_proposal_with_qc(round + 1, genesis_qc, &signer);
    let a2 = test_utils::make_proposal_with_parent_and_overrides(
        round + 3,
        round + 3,
        &a1,
        None,
        &signer,
        Some(21),
    );
    safety_rules.initialize(&proof).unwrap();
    safety_rules.update(a1.block().quorum_cert()).unwrap();

    assert_eq!(
        safety_rules.construct_and_sign_vote(&a2),
        Err(Error::IncorrectEpoch(21, 1))
    );
}

fn test_voting_potential_commit_id(func: RoundCallback) {
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

    let (proof, genesis_qc) = make_genesis::<Round>(&signer);
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
