// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::chained_bft::{
    common::Author,
    consensus_types::{block::Block, quorum_cert::QuorumCert},
    liveness::{
        proposer_election::{ProposalInfo, ProposerElection},
        rotating_proposer_election::RotatingProposer,
    },
};
use channel;
use futures::{executor::block_on, StreamExt};
use std::sync::Arc;
use types::validator_signer::ValidatorSigner;

#[test]
fn test_rotating_proposer() {
    let chosen_validator_signer = ValidatorSigner::random();
    let chosen_author = chosen_validator_signer.author();
    let another_validator_signer = ValidatorSigner::random();
    let another_author = another_validator_signer.author();
    let proposers = vec![chosen_author, another_author];
    let (winning_proposals_sender, winning_proposals_receiver) = channel::new_test(1_024);
    let pe = Arc::new(RotatingProposer::<u32, Author>::new(
        proposers,
        1,
        winning_proposals_sender,
    ));

    // Send a proposal from both chosen author and another author, the only winning proposals
    // follow the round-robin rotation.

    // Test genesis and the next block
    let genesis_block = Block::make_genesis_block();
    let quorum_cert = QuorumCert::certificate_for_genesis();

    let good_proposal = ProposalInfo {
        proposal: Block::make_block(
            &genesis_block,
            1,
            1,
            1,
            quorum_cert.clone(),
            &another_validator_signer,
        ),
        proposer_info: another_author,
        timeout_certificate: None,
        highest_ledger_info: quorum_cert.clone(),
    };
    let bad_proposal = ProposalInfo {
        proposal: Block::make_block(
            &genesis_block,
            2,
            1,
            2,
            quorum_cert.clone(),
            &chosen_validator_signer,
        ),
        proposer_info: chosen_author,
        timeout_certificate: None,
        highest_ledger_info: quorum_cert.clone(),
    };
    let next_good_proposal = ProposalInfo {
        proposal: Block::make_block(
            &genesis_block,
            3,
            2,
            3,
            quorum_cert.clone(),
            &chosen_validator_signer,
        ),
        proposer_info: chosen_author,
        timeout_certificate: None,
        highest_ledger_info: quorum_cert.clone(),
    };
    block_on(async move {
        pe.process_proposal(good_proposal.clone()).await;
        pe.process_proposal(bad_proposal.clone()).await;
        pe.process_proposal(next_good_proposal.clone()).await;

        assert_eq!(
            winning_proposals_receiver.take(2).collect::<Vec<_>>().await,
            vec![good_proposal, next_good_proposal],
        );
        assert_eq!(pe.is_valid_proposer(chosen_author, 1), None);
        assert_eq!(
            pe.is_valid_proposer(another_author, 1),
            Some(another_author)
        );
        assert_eq!(pe.is_valid_proposer(chosen_author, 2), Some(chosen_author));
        assert_eq!(pe.is_valid_proposer(another_author, 2), None);
        assert_eq!(pe.get_valid_proposers(1), vec![another_author]);
        assert_eq!(pe.get_valid_proposers(2), vec![chosen_author]);
    });
}

#[test]
fn test_rotating_proposer_with_three_contiguous_rounds() {
    let chosen_validator_signer = ValidatorSigner::random();
    let chosen_author = chosen_validator_signer.author();
    let another_validator_signer = ValidatorSigner::random();
    let another_author = another_validator_signer.author();
    let proposers = vec![chosen_author, another_author];
    let (winning_proposals_sender, winning_proposals_receiver) = channel::new_test(1_024);
    let pe = Arc::new(RotatingProposer::<u32, Author>::new(
        proposers,
        3,
        winning_proposals_sender,
    ));

    // Send a proposal from both chosen author and another author, the only winning proposals
    // follow the round-robin rotation with 3 contiguous rounds.

    // Test genesis and the next block
    let genesis_block = Block::make_genesis_block();
    let quorum_cert = QuorumCert::certificate_for_genesis();

    let good_proposal = ProposalInfo {
        proposal: Block::make_block(
            &genesis_block,
            1,
            1,
            1,
            quorum_cert.clone(),
            &chosen_validator_signer,
        ),
        proposer_info: chosen_author,
        timeout_certificate: None,
        highest_ledger_info: quorum_cert.clone(),
    };
    let bad_proposal = ProposalInfo {
        proposal: Block::make_block(
            &genesis_block,
            2,
            1,
            2,
            quorum_cert.clone(),
            &another_validator_signer,
        ),
        proposer_info: another_author,
        timeout_certificate: None,
        highest_ledger_info: quorum_cert.clone(),
    };
    let next_good_proposal = ProposalInfo {
        proposal: Block::make_block(
            &genesis_block,
            3,
            2,
            3,
            quorum_cert.clone(),
            &chosen_validator_signer,
        ),
        proposer_info: chosen_author,
        timeout_certificate: None,
        highest_ledger_info: quorum_cert.clone(),
    };
    block_on(async move {
        pe.process_proposal(good_proposal.clone()).await;
        pe.process_proposal(bad_proposal.clone()).await;
        pe.process_proposal(next_good_proposal.clone()).await;

        assert_eq!(
            winning_proposals_receiver.take(2).collect::<Vec<_>>().await,
            vec![good_proposal, next_good_proposal],
        );
        assert_eq!(pe.is_valid_proposer(another_author, 1), None);
        assert_eq!(pe.is_valid_proposer(chosen_author, 1), Some(chosen_author));
        assert_eq!(pe.is_valid_proposer(chosen_author, 2), Some(chosen_author));
        assert_eq!(pe.is_valid_proposer(another_author, 2), None);
        assert_eq!(pe.get_valid_proposers(1), vec![chosen_author]);
        assert_eq!(pe.get_valid_proposers(2), vec![chosen_author]);
    });
}

#[test]
fn test_fixed_proposer() {
    let chosen_validator_signer = ValidatorSigner::random();
    let chosen_author = chosen_validator_signer.author();
    let another_validator_signer = ValidatorSigner::random();
    let another_author = another_validator_signer.author();
    let (winning_proposals_sender, winning_proposals_receiver) = channel::new_test(1_024);
    let pe = Arc::new(RotatingProposer::<u32, Author>::new(
        vec![chosen_author],
        1,
        winning_proposals_sender,
    ));

    // Send a proposal from both chosen author and another author, the only winning proposal is
    // from the chosen author.

    // Test genesis and the next block
    let genesis_block = Block::make_genesis_block();
    let quorum_cert = QuorumCert::certificate_for_genesis();

    let good_proposal = ProposalInfo {
        proposal: Block::make_block(
            &genesis_block,
            1,
            1,
            1,
            quorum_cert.clone(),
            &chosen_validator_signer,
        ),
        proposer_info: chosen_author,
        timeout_certificate: None,
        highest_ledger_info: quorum_cert.clone(),
    };
    let bad_proposal = ProposalInfo {
        proposal: Block::make_block(
            &genesis_block,
            2,
            1,
            2,
            quorum_cert.clone(),
            &another_validator_signer,
        ),
        proposer_info: another_author,
        timeout_certificate: None,
        highest_ledger_info: quorum_cert.clone(),
    };
    let next_good_proposal = ProposalInfo {
        proposal: Block::make_block(
            &genesis_block,
            2,
            2,
            3,
            quorum_cert.clone(),
            &chosen_validator_signer,
        ),
        proposer_info: chosen_author,
        timeout_certificate: None,
        highest_ledger_info: quorum_cert.clone(),
    };
    block_on(async move {
        pe.process_proposal(good_proposal.clone()).await;
        pe.process_proposal(bad_proposal.clone()).await;
        pe.process_proposal(next_good_proposal.clone()).await;

        assert_eq!(
            winning_proposals_receiver.take(2).collect::<Vec<_>>().await,
            vec![good_proposal, next_good_proposal],
        );
        assert_eq!(pe.is_valid_proposer(chosen_author, 1), Some(chosen_author));
        assert_eq!(pe.is_valid_proposer(another_author, 1), None);
        assert_eq!(pe.get_valid_proposers(1), vec![chosen_author]);
    });
}
