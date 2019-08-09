// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::chained_bft::{
    consensus_types::{
        block::Block, proposal_msg::ProposalMsg, quorum_cert::QuorumCert, sync_info::SyncInfo,
    },
    liveness::{proposer_election::ProposerElection, rotating_proposer_election::RotatingProposer},
};
use nextgen_crypto::ed25519::*;
use std::sync::Arc;
use types::validator_signer::ValidatorSigner;

#[test]
fn test_rotating_proposer() {
    let chosen_validator_signer = ValidatorSigner::<Ed25519PrivateKey>::random([0u8; 32]);
    let chosen_author = chosen_validator_signer.author();
    let another_validator_signer = ValidatorSigner::<Ed25519PrivateKey>::random([1u8; 32]);
    let another_author = another_validator_signer.author();
    let proposers = vec![chosen_author, another_author];
    let pe: Arc<dyn ProposerElection<u32, Ed25519Signature>> =
        Arc::new(RotatingProposer::new(proposers, 1));

    // Send a proposal from both chosen author and another author, the only winning proposals
    // follow the round-robin rotation.

    // Test genesis and the next block
    let genesis_block = Block::make_genesis_block();
    let quorum_cert = QuorumCert::certificate_for_genesis();

    let good_proposal = ProposalMsg {
        proposal: Block::make_block(
            &genesis_block,
            1,
            1,
            1,
            quorum_cert.clone(),
            &another_validator_signer,
        ),
        sync_info: SyncInfo::new(quorum_cert.clone(), quorum_cert.clone(), None),
    };
    let bad_proposal = ProposalMsg {
        proposal: Block::make_block(
            &genesis_block,
            2,
            1,
            2,
            quorum_cert.clone(),
            &chosen_validator_signer,
        ),
        sync_info: SyncInfo::new(quorum_cert.clone(), quorum_cert.clone(), None),
    };
    let next_good_proposal = ProposalMsg {
        proposal: Block::make_block(
            &genesis_block,
            3,
            2,
            3,
            quorum_cert.clone(),
            &chosen_validator_signer,
        ),
        sync_info: SyncInfo::new(quorum_cert.clone(), quorum_cert.clone(), None),
    };
    assert_eq!(
        pe.process_proposal(good_proposal.clone()),
        Some(good_proposal)
    );
    assert_eq!(pe.process_proposal(bad_proposal.clone()), None);
    assert_eq!(
        pe.process_proposal(next_good_proposal.clone()),
        Some(next_good_proposal)
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
}

#[test]
fn test_rotating_proposer_with_three_contiguous_rounds() {
    let chosen_validator_signer = ValidatorSigner::<Ed25519PrivateKey>::random([0u8; 32]);
    let chosen_author = chosen_validator_signer.author();
    let another_validator_signer = ValidatorSigner::<Ed25519PrivateKey>::random([1u8; 32]);
    let another_author = another_validator_signer.author();
    let proposers = vec![chosen_author, another_author];
    let pe: Arc<dyn ProposerElection<u32, Ed25519Signature>> =
        Arc::new(RotatingProposer::new(proposers, 3));

    // Send a proposal from both chosen author and another author, the only winning proposals
    // follow the round-robin rotation with 3 contiguous rounds.

    // Test genesis and the next block
    let genesis_block = Block::make_genesis_block();
    let quorum_cert = QuorumCert::certificate_for_genesis();

    let good_proposal = ProposalMsg {
        proposal: Block::make_block(
            &genesis_block,
            1,
            1,
            1,
            quorum_cert.clone(),
            &chosen_validator_signer,
        ),
        sync_info: SyncInfo::new(quorum_cert.clone(), quorum_cert.clone(), None),
    };
    let bad_proposal = ProposalMsg {
        proposal: Block::make_block(
            &genesis_block,
            2,
            1,
            2,
            quorum_cert.clone(),
            &another_validator_signer,
        ),
        sync_info: SyncInfo::new(quorum_cert.clone(), quorum_cert.clone(), None),
    };
    let next_good_proposal = ProposalMsg {
        proposal: Block::make_block(
            &genesis_block,
            3,
            2,
            3,
            quorum_cert.clone(),
            &chosen_validator_signer,
        ),
        sync_info: SyncInfo::new(quorum_cert.clone(), quorum_cert.clone(), None),
    };
    assert_eq!(
        pe.process_proposal(good_proposal.clone()),
        Some(good_proposal)
    );
    assert_eq!(pe.process_proposal(bad_proposal.clone()), None);
    assert_eq!(
        pe.process_proposal(next_good_proposal.clone()),
        Some(next_good_proposal)
    );
    assert_eq!(pe.is_valid_proposer(another_author, 1), None);
    assert_eq!(pe.is_valid_proposer(chosen_author, 1), Some(chosen_author));
    assert_eq!(pe.is_valid_proposer(chosen_author, 2), Some(chosen_author));
    assert_eq!(pe.is_valid_proposer(another_author, 2), None);
    assert_eq!(pe.get_valid_proposers(1), vec![chosen_author]);
    assert_eq!(pe.get_valid_proposers(2), vec![chosen_author]);
}

#[test]
fn test_fixed_proposer() {
    let chosen_validator_signer = ValidatorSigner::<Ed25519PrivateKey>::random([0u8; 32]);
    let chosen_author = chosen_validator_signer.author();
    let another_validator_signer = ValidatorSigner::<Ed25519PrivateKey>::random([1u8; 32]);
    let another_author = another_validator_signer.author();
    let pe: Arc<dyn ProposerElection<u32, Ed25519Signature>> =
        Arc::new(RotatingProposer::new(vec![chosen_author], 1));

    // Send a proposal from both chosen author and another author, the only winning proposal is
    // from the chosen author.

    // Test genesis and the next block
    let genesis_block = Block::make_genesis_block();
    let quorum_cert = QuorumCert::certificate_for_genesis();

    let good_proposal = ProposalMsg {
        proposal: Block::make_block(
            &genesis_block,
            1,
            1,
            1,
            quorum_cert.clone(),
            &chosen_validator_signer,
        ),
        sync_info: SyncInfo::new(quorum_cert.clone(), quorum_cert.clone(), None),
    };
    let bad_proposal = ProposalMsg {
        proposal: Block::make_block(
            &genesis_block,
            2,
            1,
            2,
            quorum_cert.clone(),
            &another_validator_signer,
        ),
        sync_info: SyncInfo::new(quorum_cert.clone(), quorum_cert.clone(), None),
    };
    let next_good_proposal = ProposalMsg {
        proposal: Block::make_block(
            &genesis_block,
            2,
            2,
            3,
            quorum_cert.clone(),
            &chosen_validator_signer,
        ),
        sync_info: SyncInfo::new(quorum_cert.clone(), quorum_cert.clone(), None),
    };
    assert_eq!(
        pe.process_proposal(good_proposal.clone()),
        Some(good_proposal)
    );
    assert_eq!(pe.process_proposal(bad_proposal.clone()), None);
    assert_eq!(
        pe.process_proposal(next_good_proposal.clone()),
        Some(next_good_proposal)
    );
    assert_eq!(pe.is_valid_proposer(chosen_author, 1), Some(chosen_author));
    assert_eq!(pe.is_valid_proposer(another_author, 1), None);
    assert_eq!(pe.get_valid_proposers(1), vec![chosen_author]);
}
