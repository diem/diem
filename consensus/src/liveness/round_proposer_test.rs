// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::liveness::{
    proposer_election::ProposerElection, round_proposer_election::RoundProposer,
};
use consensus_types::block::{block_test_utils::certificate_for_genesis, Block};
use diem_types::validator_signer::ValidatorSigner;

use consensus_types::common::{Author, Round};
use std::collections::HashMap;

#[test]
fn test_round_proposer() {
    let chosen_validator_signer_round1 = ValidatorSigner::random([0u8; 32]);
    let chosen_author_round1 = chosen_validator_signer_round1.author();
    let chosen_validator_signer_round2 = ValidatorSigner::random([0u8; 32]);
    let chosen_author_round2 = chosen_validator_signer_round2.author();
    let another_validator_signer = ValidatorSigner::random([1u8; 32]);
    let another_author = another_validator_signer.author();

    // A map that specifies the proposer per round
    let mut round_proposers: HashMap<Round, Author> = HashMap::new();
    round_proposers.insert(1, chosen_author_round1);
    round_proposers.insert(2, chosen_author_round2);

    let pe: Box<dyn ProposerElection> =
        Box::new(RoundProposer::new(round_proposers, chosen_author_round1));

    // Send a proposal from both chosen author and another author, the only winning proposals
    // follow the round-proposers mapping

    let quorum_cert = certificate_for_genesis();

    let good_proposal = Block::new_proposal(
        vec![],
        1,
        1,
        quorum_cert.clone(),
        &chosen_validator_signer_round1,
    );
    let bad_proposal =
        Block::new_proposal(vec![], 1, 2, quorum_cert.clone(), &another_validator_signer);
    let next_good_proposal = Block::new_proposal(
        vec![],
        2,
        3,
        quorum_cert.clone(),
        &chosen_validator_signer_round2,
    );
    // In round 3, send a proposal from chosen_author_round1 (which is also the default proposer).
    // The proposal should win because the map doesn't specify proposer for round 3 hence
    // falling back on the default proposer
    let next_next_good_proposal =
        Block::new_proposal(vec![], 3, 4, quorum_cert, &chosen_validator_signer_round1);

    assert!(pe.is_valid_proposal(&good_proposal));
    assert!(!pe.is_valid_proposal(&bad_proposal));
    assert!(pe.is_valid_proposal(&next_good_proposal),);
    assert!(pe.is_valid_proposal(&next_next_good_proposal),);
    assert!(pe.is_valid_proposer(chosen_author_round1, 1),);
    assert!(!pe.is_valid_proposer(another_author, 1));
    assert!(pe.is_valid_proposer(chosen_author_round2, 2));
    assert!(!pe.is_valid_proposer(another_author, 2));
    assert!(pe.is_valid_proposer(chosen_author_round1, 3));
    assert!(!pe.is_valid_proposer(another_author, 3));
    assert_eq!(pe.get_valid_proposer(1), chosen_author_round1);
    assert_eq!(pe.get_valid_proposer(2), chosen_author_round2);
    assert_eq!(pe.get_valid_proposer(3), chosen_author_round1);
}
