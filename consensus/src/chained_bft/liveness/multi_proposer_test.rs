// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::chained_bft::{
    consensus_types::{block::Block, quorum_cert::QuorumCert},
    liveness::{
        multi_proposer_election::{self, MultiProposer},
        proposer_election::ProposerElection,
    },
};
use crypto::ed25519::*;
use types::validator_signer::ValidatorSigner;

#[test]
fn test_multi_proposer() {
    let mut signers = vec![];
    let mut proposers = vec![];
    for i in 0..8 {
        let signer = ValidatorSigner::<Ed25519PrivateKey>::random([i; 32]);
        proposers.push(signer.author());
        signers.push(signer);
    }
    let mut pe: Box<dyn ProposerElection<u32>> = Box::new(MultiProposer::new(proposers.clone(), 2));
    let round = 1;
    let first_hash = multi_proposer_election::hash(round);
    let primary_idx = (first_hash % 8) as usize;
    let second_hash = multi_proposer_election::hash(first_hash);
    let secondary_idx = (second_hash % 7) as usize; // assuming no collisions in this case

    let primary_proposer = proposers[primary_idx];
    let secondary_proposer = proposers[secondary_idx];
    assert_eq!(
        pe.get_valid_proposers(round),
        vec![primary_proposer, secondary_proposer]
    );
    assert_eq!(
        pe.is_valid_proposer(primary_proposer, round),
        Some(primary_proposer)
    );
    assert_eq!(
        pe.is_valid_proposer(secondary_proposer, round),
        Some(secondary_proposer)
    );

    let mut another_idx = (primary_idx + 1) % 8;
    if another_idx == secondary_idx {
        another_idx = (another_idx + 1) % 8;
    }
    assert_eq!(pe.is_valid_proposer(proposers[another_idx], round), None);

    let genesis_block = Block::make_genesis_block();
    let good_proposal = Block::make_block(
        &genesis_block,
        1,
        1,
        1,
        QuorumCert::certificate_for_genesis(),
        &signers[primary_idx],
    );
    assert_eq!(pe.take_backup_proposal(1), None);
    assert_eq!(
        pe.process_proposal(good_proposal.clone()),
        Some(good_proposal)
    );
    assert_eq!(pe.take_backup_proposal(1), None);

    let secondary_proposal = Block::make_block(
        &genesis_block,
        1,
        1,
        1,
        QuorumCert::certificate_for_genesis(),
        &signers[secondary_idx],
    );
    assert_eq!(pe.process_proposal(secondary_proposal.clone()), None);
    assert_eq!(pe.take_backup_proposal(2), None);
    assert_eq!(pe.take_backup_proposal(1), Some(secondary_proposal));
    // has been already popped out
    assert_eq!(pe.take_backup_proposal(1), None);
}

#[test]
fn test_multi_proposer_take_all() {
    // In case num of proposers per round is equal to the overall num of proposers
    // all the proposers are valid candidates
    let mut signers = vec![];
    let mut proposers = vec![];
    for i in 0..8 {
        let signer = ValidatorSigner::<Ed25519PrivateKey>::random([i; 32]);
        proposers.push(signer.author());
        signers.push(signer);
    }
    let pe: Box<dyn ProposerElection<u32>> = Box::new(MultiProposer::new(proposers.clone(), 8));
    let candidates = pe.get_valid_proposers(1);
    for p in proposers {
        assert!(candidates.contains(&p));
    }
}

#[test]
fn test_multi_proposer_hash() {
    // Verify that the hash distributes primary proposers in a "reasonable" fashion
    let mut counts = vec![0; 10];
    for round in 0..10000 {
        let idx = (multi_proposer_election::hash(round) % counts.len() as u64) as usize;
        counts[idx] += 1;
    }
    for c in counts {
        assert!(c > 900);
    }
}
