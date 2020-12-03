// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::liveness::{
    proposer_election::ProposerElection, rotating_proposer_election::RotatingProposer,
};
use consensus_types::block::{block_test_utils::certificate_for_genesis, Block};
use diem_types::validator_signer::ValidatorSigner;

#[test]
fn test_rotating_proposer() {
    let chosen_validator_signer = ValidatorSigner::random([0u8; 32]);
    let chosen_author = chosen_validator_signer.author();
    let another_validator_signer = ValidatorSigner::random([1u8; 32]);
    let another_author = another_validator_signer.author();
    let proposers = vec![chosen_author, another_author];
    let pe: Box<dyn ProposerElection> = Box::new(RotatingProposer::new(proposers, 1));

    // Send a proposal from both chosen author and another author, the only winning proposals
    // follow the round-robin rotation.

    // Test genesis and the next block
    let quorum_cert = certificate_for_genesis();

    let good_proposal =
        Block::new_proposal(vec![], 1, 1, quorum_cert.clone(), &another_validator_signer);
    let bad_proposal =
        Block::new_proposal(vec![], 1, 2, quorum_cert.clone(), &chosen_validator_signer);
    let next_good_proposal =
        Block::new_proposal(vec![], 2, 3, quorum_cert, &chosen_validator_signer);
    assert!(pe.is_valid_proposal(&good_proposal));
    assert!(!pe.is_valid_proposal(&bad_proposal));
    assert!(pe.is_valid_proposal(&next_good_proposal),);
    assert!(!pe.is_valid_proposer(chosen_author, 1));
    assert!(pe.is_valid_proposer(another_author, 1),);
    assert!(pe.is_valid_proposer(chosen_author, 2));
    assert!(!pe.is_valid_proposer(another_author, 2));
    assert_eq!(pe.get_valid_proposer(1), another_author);
    assert_eq!(pe.get_valid_proposer(2), chosen_author);
}

#[test]
fn test_rotating_proposer_with_three_contiguous_rounds() {
    let chosen_validator_signer = ValidatorSigner::random([0u8; 32]);
    let chosen_author = chosen_validator_signer.author();
    let another_validator_signer = ValidatorSigner::random([1u8; 32]);
    let another_author = another_validator_signer.author();
    let proposers = vec![chosen_author, another_author];
    let pe: Box<dyn ProposerElection> = Box::new(RotatingProposer::new(proposers, 3));

    // Send a proposal from both chosen author and another author, the only winning proposals
    // follow the round-robin rotation with 3 contiguous rounds.

    // Test genesis and the next block
    let quorum_cert = certificate_for_genesis();

    let good_proposal =
        Block::new_proposal(vec![], 1, 1, quorum_cert.clone(), &chosen_validator_signer);
    let bad_proposal =
        Block::new_proposal(vec![], 1, 2, quorum_cert.clone(), &another_validator_signer);
    let next_good_proposal =
        Block::new_proposal(vec![], 2, 3, quorum_cert, &chosen_validator_signer);
    assert!(pe.is_valid_proposal(&good_proposal),);
    assert!(!pe.is_valid_proposal(&bad_proposal));
    assert!(pe.is_valid_proposal(&next_good_proposal),);
    assert!(!pe.is_valid_proposer(another_author, 1));
    assert!(pe.is_valid_proposer(chosen_author, 1));
    assert!(pe.is_valid_proposer(chosen_author, 2));
    assert!(!pe.is_valid_proposer(another_author, 2));
    assert_eq!(pe.get_valid_proposer(1), chosen_author);
    assert_eq!(pe.get_valid_proposer(2), chosen_author);
}

#[test]
fn test_fixed_proposer() {
    let chosen_validator_signer = ValidatorSigner::random([0u8; 32]);
    let chosen_author = chosen_validator_signer.author();
    let another_validator_signer = ValidatorSigner::random([1u8; 32]);
    let another_author = another_validator_signer.author();
    let pe: Box<dyn ProposerElection> = Box::new(RotatingProposer::new(vec![chosen_author], 1));

    // Send a proposal from both chosen author and another author, the only winning proposal is
    // from the chosen author.

    // Test genesis and the next block
    let quorum_cert = certificate_for_genesis();

    let good_proposal =
        Block::new_proposal(vec![], 1, 1, quorum_cert.clone(), &chosen_validator_signer);
    let bad_proposal =
        Block::new_proposal(vec![], 1, 2, quorum_cert.clone(), &another_validator_signer);
    let next_good_proposal =
        Block::new_proposal(vec![], 2, 3, quorum_cert, &chosen_validator_signer);
    assert!(pe.is_valid_proposal(&good_proposal));
    assert!(!pe.is_valid_proposal(&bad_proposal));
    assert!(pe.is_valid_proposal(&next_good_proposal));
    assert!(pe.is_valid_proposer(chosen_author, 1));
    assert!(!pe.is_valid_proposer(another_author, 1));
    assert_eq!(pe.get_valid_proposer(1), chosen_author);
}
