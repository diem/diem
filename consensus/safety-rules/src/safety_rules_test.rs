// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{ConsensusState, ProposalReject, SafetyRules};
use consensus_types::{
    block::Block, block_info::BlockInfo, common::Round, quorum_cert::QuorumCert,
    sync_info::SyncInfo, vote::Vote, vote_data::VoteData, vote_msg::VoteMsg,
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

fn make_block_with_qc(
    round: Round,
    qc: QuorumCert,
    validator_signer: &ValidatorSigner,
) -> Block<Round> {
    Block::<Round>::new_internal(
        round,
        0,
        round,
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        qc,
        validator_signer,
    )
}

fn make_block_with_parent(
    round: Round,
    parent: &Block<Round>,
    committed: Option<&Block<Round>>,
    validator_signer: &ValidatorSigner,
    highest_quorum_cert: &QuorumCert,
) -> Block<Round> {
    let vote_data = VoteData::new(
        BlockInfo::from_block(parent, HashValue::zero(), 0),
        parent.quorum_cert().certified_block().clone(),
    );

    let sync_info = SyncInfo::new(
        highest_quorum_cert.clone(),
        highest_quorum_cert.clone(),
        None,
    );

    let ledger_info = match committed {
        Some(committed) => LedgerInfo::new(
            0,
            HashValue::zero(),
            vote_data.hash(),
            committed.id(),
            committed.epoch(),
            committed.timestamp_usecs(),
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

    let vote_msg = VoteMsg::new(
        Vote::new(
            vote_data.clone(),
            validator_signer.author(),
            ledger_info,
            validator_signer,
        ),
        sync_info,
    );

    let mut ledger_info_with_signatures =
        LedgerInfoWithSignatures::new(vote_msg.vote().ledger_info().clone(), BTreeMap::new());

    vote_msg
        .vote()
        .signature()
        .clone()
        .add_to_li(vote_msg.vote().author(), &mut ledger_info_with_signatures);

    let qc = QuorumCert::new(vote_data, ledger_info_with_signatures);

    make_block_with_qc(round, qc, validator_signer)
}

#[test]
fn test_initial_state() {
    // Start from scratch, verify the state
    let block = Block::<u64>::make_genesis_block();

    let safety_rules = SafetyRules::new(ConsensusState::default());
    let state = safety_rules.consensus_state();
    assert_eq!(state.last_vote_round(), block.round());
    assert_eq!(state.preferred_block_round(), block.round());
}

#[test]
fn test_preferred_block_rule() {
    // Preferred block is the highest 2-chain head.
    let validator_signer = ValidatorSigner::from_int(0);
    let mut safety_rules = SafetyRules::new(ConsensusState::default());

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

    let a1 = make_block_with_qc(round + 1, genesis_qc.clone(), &validator_signer);
    let b1 = make_block_with_qc(round + 2, genesis_qc.clone(), &validator_signer);
    let b2 = make_block_with_parent(round + 3, &a1, None, &validator_signer, &b1.quorum_cert());
    let a2 = make_block_with_parent(round + 4, &b1, None, &validator_signer, &b2.quorum_cert());
    let b3 = make_block_with_parent(round + 5, &b2, None, &validator_signer, &a2.quorum_cert());
    let a3 = make_block_with_parent(round + 6, &a2, None, &validator_signer, &b3.quorum_cert());
    let a4 = make_block_with_parent(round + 7, &a3, None, &validator_signer, &a3.quorum_cert());

    safety_rules.update(a1.quorum_cert());
    assert_eq!(
        safety_rules.consensus_state().preferred_block_round(),
        genesis_block.round()
    );

    safety_rules.update(b1.quorum_cert());
    assert_eq!(
        safety_rules.consensus_state().preferred_block_round(),
        genesis_block.round()
    );

    safety_rules.update(a2.quorum_cert());
    assert_eq!(
        safety_rules.consensus_state().preferred_block_round(),
        genesis_block.round()
    );

    safety_rules.update(b2.quorum_cert());
    assert_eq!(
        safety_rules.consensus_state().preferred_block_round(),
        genesis_block.round()
    );

    safety_rules.update(a3.quorum_cert());
    assert_eq!(
        safety_rules.consensus_state().preferred_block_round(),
        b1.round()
    );

    safety_rules.update(b3.quorum_cert());
    assert_eq!(
        safety_rules.consensus_state().preferred_block_round(),
        b1.round()
    );

    safety_rules.update(a4.quorum_cert());
    assert_eq!(
        safety_rules.consensus_state().preferred_block_round(),
        a2.round()
    );
}

#[test]
/// Test the potential ledger info that we're going to use in case of voting
fn test_voting_potential_commit_id() {
    let validator_signer = ValidatorSigner::from_int(0);
    let mut safety_rules = SafetyRules::new(ConsensusState::default());

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

    let a1 = make_block_with_qc(round + 1, genesis_qc.clone(), &validator_signer);
    let b1 = make_block_with_qc(round + 2, genesis_qc.clone(), &validator_signer);
    let a2 = make_block_with_parent(round + 3, &a1, None, &validator_signer, &b1.quorum_cert());
    let a3 = make_block_with_parent(round + 4, &a2, None, &validator_signer, &a2.quorum_cert());
    let a4 = make_block_with_parent(
        round + 5,
        &a3,
        Some(&a2),
        &validator_signer,
        &a3.quorum_cert(),
    );
    let a5 = make_block_with_parent(
        round + 6,
        &a4,
        Some(&a3),
        &validator_signer,
        &a4.quorum_cert(),
    );

    for b in &[&a1, &b1, &a2, &a3] {
        safety_rules.update(b.quorum_cert());
        let voting_info = safety_rules.voting_rule(b).unwrap();
        assert_eq!(voting_info.potential_commit_id, None);
    }

    safety_rules.update(a4.quorum_cert());
    assert_eq!(
        safety_rules.voting_rule(&a4).unwrap().potential_commit_id,
        Some(a2.id())
    );

    safety_rules.update(a5.quorum_cert());
    assert_eq!(
        safety_rules.voting_rule(&a5).unwrap().potential_commit_id,
        Some(a3.id())
    );
}

#[test]
fn test_voting() {
    let validator_signer = ValidatorSigner::from_int(0);
    let mut safety_rules = SafetyRules::new(ConsensusState::default());

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

    let a1 = make_block_with_qc(round + 1, genesis_qc.clone(), &validator_signer);
    let b1 = make_block_with_qc(round + 2, genesis_qc.clone(), &validator_signer);
    let b2 = make_block_with_parent(round + 3, &a1, None, &validator_signer, &b1.quorum_cert());
    let a2 = make_block_with_parent(round + 4, &b1, None, &validator_signer, &b2.quorum_cert());
    let a3 = make_block_with_parent(round + 5, &a2, None, &validator_signer, &a2.quorum_cert());
    let b3 = make_block_with_parent(round + 6, &b2, None, &validator_signer, &a3.quorum_cert());
    let a4 = make_block_with_parent(round + 7, &a3, None, &validator_signer, &b3.quorum_cert());
    let b4 = make_block_with_parent(round + 8, &b2, None, &validator_signer, &a4.quorum_cert());

    safety_rules.update(a1.quorum_cert());
    let mut voting_info = safety_rules.voting_rule(&a1).unwrap();
    assert_eq!(voting_info.potential_commit_id, None);

    safety_rules.update(b1.quorum_cert());
    voting_info = safety_rules.voting_rule(&b1).unwrap();
    assert_eq!(voting_info.potential_commit_id, None);

    safety_rules.update(a2.quorum_cert());
    voting_info = safety_rules.voting_rule(&a2).unwrap();
    assert_eq!(voting_info.potential_commit_id, None);

    safety_rules.update(b2.quorum_cert());
    assert_eq!(
        safety_rules.voting_rule(&b2),
        Err(ProposalReject::OldProposal {
            last_vote_round: 4,
            proposal_round: 3,
        })
    );

    safety_rules.update(a3.quorum_cert());
    voting_info = safety_rules.voting_rule(&a3).unwrap();
    assert_eq!(voting_info.potential_commit_id, None);

    safety_rules.update(b3.quorum_cert());
    voting_info = safety_rules.voting_rule(&b3).unwrap();
    assert_eq!(voting_info.potential_commit_id, None);

    safety_rules.update(a4.quorum_cert());
    voting_info = safety_rules.voting_rule(&a4).unwrap();
    assert_eq!(voting_info.potential_commit_id, None);

    safety_rules.update(a4.quorum_cert());
    assert_eq!(
        safety_rules.voting_rule(&a4),
        Err(ProposalReject::OldProposal {
            last_vote_round: 7,
            proposal_round: 7,
        })
    );
    safety_rules.update(b4.quorum_cert());
    assert_eq!(
        safety_rules.voting_rule(&b4),
        Err(ProposalReject::ProposalRoundLowerThenPreferredBlock {
            preferred_block_round: 4,
        })
    );
}

#[test]
fn test_commit_rule_consecutive_rounds() {
    let validator_signer = ValidatorSigner::from_int(0);
    let safety_rules = SafetyRules::new(ConsensusState::default());

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

    let a1 = make_block_with_qc(round + 1, genesis_qc.clone(), &validator_signer);
    let b1 = make_block_with_qc(round + 2, genesis_qc.clone(), &validator_signer);
    let b2 = make_block_with_parent(round + 3, &b1, None, &validator_signer, &b1.quorum_cert());
    let a2 = make_block_with_parent(round + 4, &a1, None, &validator_signer, &b2.quorum_cert());
    let a3 = make_block_with_parent(round + 5, &a2, None, &validator_signer, &a2.quorum_cert());
    let a4 = make_block_with_parent(round + 6, &a3, None, &validator_signer, &a3.quorum_cert());

    assert_eq!(
        safety_rules.commit_rule_for_certified_block(a1.quorum_cert(), a1.round()),
        None
    );
    assert_eq!(
        safety_rules.commit_rule_for_certified_block(b1.quorum_cert(), b1.round()),
        None
    );
    assert_eq!(
        safety_rules.commit_rule_for_certified_block(b2.quorum_cert(), b2.round()),
        None
    );
    assert_eq!(
        safety_rules.commit_rule_for_certified_block(a2.quorum_cert(), a2.round()),
        None
    );
    assert_eq!(
        safety_rules.commit_rule_for_certified_block(a3.quorum_cert(), a3.round()),
        None
    );
    assert_eq!(
        safety_rules.commit_rule_for_certified_block(a4.quorum_cert(), a4.round()),
        Some(a2.id())
    );
}
