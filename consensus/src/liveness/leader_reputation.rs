// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::liveness::proposer_election::{next, ProposerElection};
use consensus_types::{
    block::Block,
    common::{Author, Round},
};
use diem_crypto::HashValue;
use diem_infallible::Mutex;
use diem_logger::prelude::*;
use diem_types::block_metadata::{new_block_event_key, NewBlockEvent};
use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
    sync::Arc,
};
use storage_interface::{DbReader, Order};

/// Interface to query committed BlockMetadata.
pub trait MetadataBackend: Send + Sync {
    /// Return a contiguous BlockMetadata window in which last one is at target_round or
    /// latest committed, return all previous one if not enough.
    fn get_block_metadata(&self, target_round: Round) -> Vec<NewBlockEvent>;
}

pub struct DiemDBBackend {
    window_size: usize,
    diem_db: Arc<dyn DbReader>,
    window: Mutex<Vec<(u64, NewBlockEvent)>>,
}

impl DiemDBBackend {
    pub fn new(window_size: usize, diem_db: Arc<dyn DbReader>) -> Self {
        Self {
            window_size,
            diem_db,
            window: Mutex::new(vec![]),
        }
    }

    fn refresh_window(&self, target_round: Round) -> anyhow::Result<()> {
        // assumes target round is not too far from latest commit
        let buffer = 10;
        let events = self.diem_db.get_events(
            &new_block_event_key(),
            u64::max_value(),
            Order::Descending,
            self.window_size as u64 + buffer,
        )?;
        let mut result = vec![];
        for (v, e) in events {
            let e = lcs::from_bytes::<NewBlockEvent>(e.event_data())?;
            if e.round() <= target_round && result.len() < self.window_size {
                result.push((v, e));
            }
        }
        *self.window.lock() = result;
        Ok(())
    }
}

impl MetadataBackend for DiemDBBackend {
    // assume the target_round only increases
    fn get_block_metadata(&self, target_round: Round) -> Vec<NewBlockEvent> {
        let (known_version, known_round) = self
            .window
            .lock()
            .first()
            .map(|(v, e)| (*v, e.round()))
            .unwrap_or((0, 0));
        if !(known_round == target_round
            || known_version == self.diem_db.get_latest_version().unwrap_or(0))
        {
            if let Err(e) = self.refresh_window(target_round) {
                error!(
                    error = ?e, "[leader reputation] Fail to refresh window",
                );
                return vec![];
            }
        }
        self.window
            .lock()
            .clone()
            .into_iter()
            .map(|(_, e)| e)
            .collect()
    }
}

/// Interface to calculate weights for proposers based on history.
pub trait ReputationHeuristic: Send + Sync {
    /// Return the weights of all candidates based on the history.
    fn get_weights(&self, candidates: &[Author], history: &[NewBlockEvent]) -> Vec<u64>;
}

/// If candidate appear in the history, it's assigned active_weight otherwise inactive weight.
pub struct ActiveInactiveHeuristic {
    active_weight: u64,
    inactive_weight: u64,
}

impl ActiveInactiveHeuristic {
    pub fn new(active_weight: u64, inactive_weight: u64) -> Self {
        Self {
            active_weight,
            inactive_weight,
        }
    }
}

impl ReputationHeuristic for ActiveInactiveHeuristic {
    fn get_weights(&self, candidates: &[Author], history: &[NewBlockEvent]) -> Vec<u64> {
        let set = history.iter().fold(HashSet::new(), |mut set, meta| {
            set.insert(meta.proposer());
            set.extend(meta.votes().into_iter());
            set
        });
        candidates
            .iter()
            .map(|author| {
                if set.contains(&author) {
                    self.active_weight
                } else {
                    self.inactive_weight
                }
            })
            .collect()
    }
}

/// Committed history based proposer election implementation that could help bias towards
/// successful leaders to help improve performance.
pub struct LeaderReputation {
    proposers: Vec<Author>,
    backend: Box<dyn MetadataBackend>,
    heuristic: Box<dyn ReputationHeuristic>,
    already_proposed: Mutex<(Round, HashMap<Author, HashValue>)>,
}

impl LeaderReputation {
    pub fn new(
        proposers: Vec<Author>,
        backend: Box<dyn MetadataBackend>,
        heuristic: Box<dyn ReputationHeuristic>,
    ) -> Self {
        Self {
            proposers,
            backend,
            heuristic,
            already_proposed: Mutex::new((0, HashMap::new())),
        }
    }
}

impl ProposerElection for LeaderReputation {
    fn get_valid_proposer(&self, round: Round) -> Author {
        // TODO: configure the round gap
        let target_round = if round >= 4 { round - 4 } else { 0 };
        let sliding_window = self.backend.get_block_metadata(target_round);
        let mut weights = self.heuristic.get_weights(&self.proposers, &sliding_window);
        assert_eq!(weights.len(), self.proposers.len());
        let mut total_weight = 0;
        for w in &mut weights {
            total_weight += *w;
            *w = total_weight;
        }
        let mut state = round.to_le_bytes().to_vec();
        let chosen_weight = next(&mut state) % total_weight;
        let chosen_index = weights
            .binary_search_by(|w| {
                if *w <= chosen_weight {
                    Ordering::Less
                } else {
                    Ordering::Greater
                }
            })
            .unwrap_err();
        self.proposers[chosen_index]
    }

    /// This function will return true for at most one proposal per valid proposer for a given round.
    fn is_valid_proposal(&self, block: &Block) -> bool {
        block.author().map_or(false, |author| {
            let valid = self.is_valid_proposer(author, block.round());
            let mut already_proposed = self.already_proposed.lock();
            if !valid {
                return false;
            }
            // detect if the leader proposes more than once in this round
            match block.round().cmp(&already_proposed.0) {
                Ordering::Greater => {
                    already_proposed.0 = block.round();
                    already_proposed.1.clear();
                    already_proposed.1.insert(author, block.id());
                    true
                }
                Ordering::Equal => {
                    if already_proposed
                        .1
                        .get(&author)
                        .map_or(false, |id| *id != block.id())
                    {
                        error!(
                            SecurityEvent::InvalidConsensusProposal,
                            "Multiple proposals from {} for round {}",
                            author,
                            block.round()
                        );
                        false
                    } else {
                        already_proposed.1.insert(author, block.id());
                        true
                    }
                }
                Ordering::Less => false,
            }
        })
    }
}
