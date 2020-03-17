// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::chained_bft::liveness::{
    multi_proposer_election::{self, MultiProposer},
    proposer_election::ProposerElection,
};
use consensus_types::block::{block_test_utils::certificate_for_genesis, Block};
use libra_types::validator_signer::ValidatorSigner;

#[test]
fn test_multi_proposer() {
    let epoch = 0u64;
    let mut signers = vec![];
    let mut proposers = vec![];
    for i in 0..8 {
        let signer = ValidatorSigner::random([i; 32]);
        proposers.push(signer.author());
        signers.push(signer);
    }
    let mut pe: Box<dyn ProposerElection<u32>> =
        Box::new(MultiProposer::new(epoch, proposers.clone(), 2));
    let round = 1u64;
    let mut state = epoch.to_le_bytes().to_vec();
    state.extend_from_slice(&round.to_le_bytes());
    let primary_idx = multi_proposer_election::next(&mut state);
    let primary_idx = (primary_idx % 8) as usize;
    let secondary_idx = multi_proposer_election::next(&mut state);
    let secondary_idx = (secondary_idx % 7) as usize; // assuming no collisions in this case

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

    let good_proposal =
        Block::new_proposal(1, 1, 1, certificate_for_genesis(), &signers[primary_idx]);
    assert_eq!(pe.take_backup_proposal(1), None);
    assert_eq!(
        pe.process_proposal(good_proposal.clone()),
        Some(good_proposal)
    );
    assert_eq!(pe.take_backup_proposal(1), None);

    let secondary_proposal =
        Block::new_proposal(1, 1, 1, certificate_for_genesis(), &signers[secondary_idx]);
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
    let epoch = 0u64;
    let mut signers = vec![];
    let mut proposers = vec![];
    for i in 0..8 {
        let signer = ValidatorSigner::random([i; 32]);
        proposers.push(signer.author());
        signers.push(signer);
    }
    let pe: Box<dyn ProposerElection<u32>> =
        Box::new(MultiProposer::new(epoch, proposers.clone(), 8));
    let candidates = pe.get_valid_proposers(1);
    for p in proposers {
        assert!(candidates.contains(&p));
    }
}

#[test]
fn test_multi_proposer_hash() {
    // Verify that the hash distributes primary proposers in a "reasonable" fashion
    let mut counts = vec![0; 10];
    let epoch = 0u64;
    for round in 0..10000 {
        let mut state = epoch.to_le_bytes().to_vec();
        state.extend_from_slice(&(round as u64).to_le_bytes());
        let idx = multi_proposer_election::next(&mut state) % (counts.len() as u64);
        counts[idx as usize] += 1;
    }
    for c in counts {
        assert!(c > 900);
    }
}
