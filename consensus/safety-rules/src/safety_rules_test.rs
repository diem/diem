// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{ConsensusState, Error, SafetyRules};
use consensus_types::{
    block::Block, block_info::BlockInfo, common::Round, quorum_cert::QuorumCert, vote::Vote,
    vote_data::VoteData, vote_proposal::VoteProposal,
};
use libra_crypto::hash::{CryptoHash, HashValue};
use libra_types::{
    crypto_proxies::ValidatorSigner,
    ledger_info::{LedgerInfo, LedgerInfoWithSignatures},
};
use std::{
    collections::BTreeMap,
    time::{SystemTime, UNIX_EPOCH},
};

fn make_proposal_with_qc(
    round: Round,
    qc: QuorumCert,
    validator_signer: &ValidatorSigner,
) -> VoteProposal<Round> {
    VoteProposal::<Round>::new(
        Block::<Round>::new_proposal(
            round,
            round,
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            qc,
            validator_signer,
        ),
        HashValue::zero(),
        0,
        None,
    )
}

fn make_proposal_with_parent(
    round: Round,
    parent: &Block<Round>,
    committed: Option<&VoteProposal<Round>>,
    validator_signer: &ValidatorSigner,
) -> VoteProposal<Round> {
    let vote_data = VoteData::new(
        BlockInfo::from_block(parent, HashValue::zero(), 0, None),
        parent.quorum_cert().certified_block().clone(),
    );

    let ledger_info = match committed {
        Some(committed) => LedgerInfo::new(
            committed.version(),
            committed.executed_state_id(),
            vote_data.hash(),
            committed.block().id(),
            committed.block().epoch(),
            committed.block().timestamp_usecs(),
            None,
        ),
        None => LedgerInfo::new(
            0,
            HashValue::zero(),
            vote_data.hash(),
            HashValue::zero(),
            0,
            0,
            None,
        ),
    };

    let vote = Vote::new(
        vote_data.clone(),
        validator_signer.author(),
        ledger_info,
        validator_signer,
    );

    let mut ledger_info_with_signatures =
        LedgerInfoWithSignatures::new(vote.ledger_info().clone(), BTreeMap::new());

    vote.signature()
        .clone()
        .add_to_li(vote.author(), &mut ledger_info_with_signatures);

    let qc = QuorumCert::new(vote_data, ledger_info_with_signatures);

    make_proposal_with_qc(round, qc, validator_signer)
}

#[test]
fn test_initial_state() {
    // Start from scratch, verify the state
    let block = Block::<Round>::make_genesis_block();

    let safety_rules = SafetyRules::new(ConsensusState::default(), ValidatorSigner::from_int(0));
    let state = safety_rules.consensus_state();
    assert_eq!(state.last_vote_round(), block.round());
    assert_eq!(state.preferred_block_round(), block.round());
}

#[test]
fn test_preferred_block_rule() {
    // Preferred block is the highest 2-chain head.
    let validator_signer = ValidatorSigner::from_int(0);
    let mut safety_rules = SafetyRules::new(ConsensusState::default(), validator_signer.clone());

    // build a tree of the following form:
    //             _____    _____
    //            /     \  /     \
    // genesis---a1  b1  b2  a2  b3  a3---a4
    //         \_____/ \_____/ \_____/
    //
    // PB should change from genesis to b1 and then a2.
    let genesis_block = Block::<Round>::make_genesis_block();
    let genesis_qc = QuorumCert::certificate_for_genesis();
    let round = genesis_block.round();

    let a1 = make_proposal_with_qc(round + 1, genesis_qc.clone(), &validator_signer);
    let b1 = make_proposal_with_qc(round + 2, genesis_qc.clone(), &validator_signer);
    let b2 = make_proposal_with_parent(round + 3, a1.block(), None, &validator_signer);
    let a2 = make_proposal_with_parent(round + 4, b1.block(), None, &validator_signer);
    let b3 = make_proposal_with_parent(round + 5, b2.block(), None, &validator_signer);
    let a3 = make_proposal_with_parent(round + 6, a2.block(), None, &validator_signer);
    let a4 = make_proposal_with_parent(round + 7, a3.block(), None, &validator_signer);

    safety_rules.update(a1.block().quorum_cert());
    assert_eq!(
        safety_rules.consensus_state().preferred_block_round(),
        genesis_block.round()
    );

    safety_rules.update(b1.block().quorum_cert());
    assert_eq!(
        safety_rules.consensus_state().preferred_block_round(),
        genesis_block.round()
    );

    safety_rules.update(a2.block().quorum_cert());
    assert_eq!(
        safety_rules.consensus_state().preferred_block_round(),
        genesis_block.round()
    );

    safety_rules.update(b2.block().quorum_cert());
    assert_eq!(
        safety_rules.consensus_state().preferred_block_round(),
        genesis_block.round()
    );

    safety_rules.update(a3.block().quorum_cert());
    assert_eq!(
        safety_rules.consensus_state().preferred_block_round(),
        b1.block().round()
    );

    safety_rules.update(b3.block().quorum_cert());
    assert_eq!(
        safety_rules.consensus_state().preferred_block_round(),
        b1.block().round()
    );

    safety_rules.update(a4.block().quorum_cert());
    assert_eq!(
        safety_rules.consensus_state().preferred_block_round(),
        a2.block().round()
    );
}

#[test]
/// Test the potential ledger info that we're going to use in case of voting
fn test_voting_potential_commit_id() {
    let validator_signer = ValidatorSigner::from_int(0);
    let mut safety_rules = SafetyRules::new(ConsensusState::default(), validator_signer.clone());

    // build a tree of the following form:
    //            _____
    //           /     \
    // genesis--a1  b1  a2--a3--a4--a5
    //        \_____/
    //
    // All the votes before a4 cannot produce any potential commits.
    // A potential commit for proposal a4 is a2, a potential commit for proposal a5 is a3.
    let genesis_block = Block::<Round>::make_genesis_block();
    let genesis_qc = QuorumCert::certificate_for_genesis();
    let round = genesis_block.round();

    let a1 = make_proposal_with_qc(round + 1, genesis_qc.clone(), &validator_signer);
    let b1 = make_proposal_with_qc(round + 2, genesis_qc.clone(), &validator_signer);
    let a2 = make_proposal_with_parent(round + 3, a1.block(), None, &validator_signer);
    let a3 = make_proposal_with_parent(round + 4, a2.block(), None, &validator_signer);
    let a4 = make_proposal_with_parent(round + 5, a3.block(), Some(&a2), &validator_signer);
    let a5 = make_proposal_with_parent(round + 6, a4.block(), Some(&a3), &validator_signer);

    for b in &[&a1, &b1, &a2, &a3] {
        safety_rules.update(b.block().quorum_cert());
        let vote = safety_rules.construct_and_sign_vote(b).unwrap();
        assert_eq!(vote.ledger_info().consensus_block_id(), HashValue::zero());
    }

    safety_rules.update(a4.block().quorum_cert());
    assert_eq!(
        safety_rules
            .construct_and_sign_vote(&a4)
            .unwrap()
            .ledger_info()
            .consensus_block_id(),
        a2.block().id(),
    );

    safety_rules.update(a5.block().quorum_cert());
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
    let mut safety_rules = SafetyRules::new(ConsensusState::default(), validator_signer.clone());

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
    let genesis_qc = QuorumCert::certificate_for_genesis();
    let round = genesis_block.round();

    let a1 = make_proposal_with_qc(round + 1, genesis_qc.clone(), &validator_signer);
    let b1 = make_proposal_with_qc(round + 2, genesis_qc.clone(), &validator_signer);
    let b2 = make_proposal_with_parent(round + 3, a1.block(), None, &validator_signer);
    let a2 = make_proposal_with_parent(round + 4, b1.block(), None, &validator_signer);
    let a3 = make_proposal_with_parent(round + 5, a2.block(), None, &validator_signer);
    let b3 = make_proposal_with_parent(round + 6, b2.block(), None, &validator_signer);
    let a4 = make_proposal_with_parent(round + 7, a3.block(), None, &validator_signer);
    let b4 = make_proposal_with_parent(round + 8, b2.block(), None, &validator_signer);

    safety_rules.update(a1.block().quorum_cert());
    let mut vote = safety_rules.construct_and_sign_vote(&a1).unwrap();
    assert_eq!(vote.ledger_info().consensus_block_id(), HashValue::zero());

    safety_rules.update(b1.block().quorum_cert());
    vote = safety_rules.construct_and_sign_vote(&b1).unwrap();
    assert_eq!(vote.ledger_info().consensus_block_id(), HashValue::zero());

    safety_rules.update(a2.block().quorum_cert());
    vote = safety_rules.construct_and_sign_vote(&a2).unwrap();
    assert_eq!(vote.ledger_info().consensus_block_id(), HashValue::zero());

    safety_rules.update(b2.block().quorum_cert());
    assert_eq!(
        safety_rules.construct_and_sign_vote(&b2),
        Err(Error::OldProposal {
            last_vote_round: 4,
            proposal_round: 3,
        })
    );

    safety_rules.update(a3.block().quorum_cert());
    vote = safety_rules.construct_and_sign_vote(&a3).unwrap();
    assert_eq!(vote.ledger_info().consensus_block_id(), HashValue::zero());

    safety_rules.update(b3.block().quorum_cert());
    vote = safety_rules.construct_and_sign_vote(&b3).unwrap();
    assert_eq!(vote.ledger_info().consensus_block_id(), HashValue::zero());

    safety_rules.update(a4.block().quorum_cert());
    vote = safety_rules.construct_and_sign_vote(&a4).unwrap();
    assert_eq!(vote.ledger_info().consensus_block_id(), HashValue::zero());

    safety_rules.update(a4.block().quorum_cert());
    assert_eq!(
        safety_rules.construct_and_sign_vote(&a4),
        Err(Error::OldProposal {
            last_vote_round: 7,
            proposal_round: 7,
        })
    );
    safety_rules.update(b4.block().quorum_cert());
    assert_eq!(
        safety_rules.construct_and_sign_vote(&b4),
        Err(Error::ProposalRoundLowerThenPreferredBlock {
            preferred_block_round: 4,
        })
    );
}

#[test]
fn test_commit_rule_consecutive_rounds() {
    let validator_signer = ValidatorSigner::from_int(0);
    let safety_rules = SafetyRules::new(ConsensusState::default(), validator_signer.clone());

    // build a tree of the following form:
    //             ___________
    //            /           \
    // genesis---a1  b1---b2   a2---a3---a4
    //         \_____/
    //
    // a1 cannot be committed after a3 gathers QC because a1 and a2 are not consecutive
    // a2 can be committed after a4 gathers QC
    let genesis_block = Block::<Round>::make_genesis_block();
    let genesis_qc = QuorumCert::certificate_for_genesis();
    let round = genesis_block.round();

    let a1 = make_proposal_with_qc(round + 1, genesis_qc.clone(), &validator_signer);
    let b1 = make_proposal_with_qc(round + 2, genesis_qc.clone(), &validator_signer);
    let b2 = make_proposal_with_parent(round + 3, b1.block(), None, &validator_signer);
    let a2 = make_proposal_with_parent(round + 4, a1.block(), None, &validator_signer);
    let a3 = make_proposal_with_parent(round + 5, a2.block(), None, &validator_signer);
    let a4 = make_proposal_with_parent(round + 6, a3.block(), None, &validator_signer);

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
