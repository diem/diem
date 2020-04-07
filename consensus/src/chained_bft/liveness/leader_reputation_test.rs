// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::chained_bft::{
    liveness::{
        leader_reputation::{
            ActiveInactiveHeuristic, LeaderReputation, MetadataBackend, ReputationHeuristic,
        },
        proposer_election::{next, ProposerElection},
    },
    test_utils::TestPayload,
};
use consensus_types::{
    block::{block_test_utils::certificate_for_genesis, Block},
    common::{Author, Round},
};
use libra_types::{block_metadata::NewBlockEvent, validator_signer::ValidatorSigner};

struct MockHistory {
    window_size: usize,
    data: Vec<NewBlockEvent>,
}

impl MockHistory {
    fn new(window_size: usize, data: Vec<NewBlockEvent>) -> Self {
        Self { window_size, data }
    }
}

impl MetadataBackend for MockHistory {
    fn get_block_metadata(&self, _target_round: Round) -> Vec<NewBlockEvent> {
        let start = if self.data.len() > self.window_size {
            self.data.len() - self.window_size
        } else {
            0
        };
        self.data[start..].to_vec()
    }
}

fn create_block(proposer: Author, voters: Vec<&ValidatorSigner>) -> NewBlockEvent {
    NewBlockEvent::new(0, proposer, voters.iter().map(|v| v.author()).collect(), 0)
}

#[test]
fn test_simple_heuristic() {
    let active_weight = 9;
    let inactive_weight = 1;
    let mut proposers = vec![];
    let mut signers = vec![];
    for i in 0..8 {
        let signer = ValidatorSigner::random([i; 32]);
        proposers.push(signer.author());
        signers.push(signer);
    }
    let heuristic = ActiveInactiveHeuristic::new(active_weight, inactive_weight);
    // 1. Window size not enough
    let weights = heuristic.get_weights(&proposers, &[]);
    assert_eq!(weights.len(), proposers.len());
    for w in weights {
        assert_eq!(w, inactive_weight);
    }
    // 2. Sliding window with [proposer 0, voters 1, 2], [proposer 0, voters 3]
    let weights = heuristic.get_weights(
        &proposers,
        &[
            create_block(proposers[0], vec![&signers[1], &signers[2]]),
            create_block(proposers[0], vec![&signers[3]]),
        ],
    );
    assert_eq!(weights.len(), proposers.len());
    for (i, w) in weights.iter().enumerate() {
        let expected = if i < 4 {
            active_weight
        } else {
            inactive_weight
        };
        assert_eq!(*w, expected);
    }
}

#[test]
fn test_api() {
    let active_weight = 9;
    let inactive_weight = 1;
    let mut proposers = vec![];
    let mut signers = vec![];
    for i in 0..5 {
        let signer = ValidatorSigner::random([i; 32]);
        proposers.push(signer.author());
        signers.push(signer);
    }
    let history = vec![
        create_block(proposers[0], vec![&signers[1], &signers[2]]),
        create_block(proposers[0], vec![&signers[3]]),
    ];
    let leader_reputation = LeaderReputation::<TestPayload>::new(
        proposers.clone(),
        Box::new(MockHistory::new(1, history)),
        Box::new(ActiveInactiveHeuristic::new(active_weight, inactive_weight)),
    );
    let round = 42u64;
    // first metadata is ignored because of window size 1
    let expected_weights = vec![
        active_weight,
        inactive_weight,
        inactive_weight,
        active_weight,
        inactive_weight,
    ];
    let sum = expected_weights.iter().fold(0, |mut s, w| {
        s += *w;
        s
    });
    let mut state = round.to_le_bytes().to_vec();
    let chosen_weight = next(&mut state) % sum;
    let mut expected_index = 0usize;
    let mut accu = 0u64;
    for (i, w) in expected_weights.iter().enumerate() {
        accu += *w;
        if accu >= chosen_weight {
            expected_index = i;
        }
    }
    let unexpected_index = (expected_index + 1) % proposers.len();
    let mut proposer_election: Box<dyn ProposerElection<TestPayload>> = Box::new(leader_reputation);
    let output = proposer_election.get_valid_proposers(round);
    assert_eq!(output.len(), 1);
    assert_eq!(output[0], proposers[expected_index]);
    assert!(proposer_election
        .is_valid_proposer(proposers[expected_index], 42)
        .is_some());
    assert!(proposer_election
        .is_valid_proposer(proposers[unexpected_index], 42)
        .is_none());
    let good_proposal = Block::new_proposal(
        vec![1],
        round,
        1,
        certificate_for_genesis(),
        &signers[expected_index],
    );
    assert!(proposer_election.process_proposal(good_proposal).is_some());
    let bad_proposal = Block::new_proposal(
        vec![1],
        round,
        1,
        certificate_for_genesis(),
        &signers[unexpected_index],
    );
    assert!(proposer_election.process_proposal(bad_proposal).is_none());
    assert!(proposer_election.take_backup_proposal(round).is_none());
}
