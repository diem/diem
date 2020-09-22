// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::liveness::proposer_election::ProposerElection;
use consensus_types::common::{Author, Round};

use futures::SinkExt;
use std::collections::{HashMap, HashSet};

/// The round proposer maps a round to author (Twins only testing)
pub struct RoundProposer {
    // A pre-defined map specifying proposers per round
    proposers: HashMap<Round, Author>,
    // Default proposer to use if proposer for a round is unspecified.
    // We hardcode this to the first proposer
    default_proposer: Author,
    // The rounds that're supposed to timeout (when it's in a different partition than the proposer)
    timeout_rounds: HashSet<Round>,
    timeout_sender: channel::Sender<Round>,
}

impl RoundProposer {
    pub fn new(
        proposers: HashMap<Round, Author>,
        default_proposer: Author,
        timeout_rounds: HashSet<Round>,
        timeout_sender: channel::Sender<Round>,
    ) -> Self {
        Self {
            proposers,
            default_proposer,
            timeout_rounds,
            timeout_sender,
        }
    }
}

impl ProposerElection for RoundProposer {
    fn get_valid_proposer(&self, round: Round) -> Author {
        let proposer = match self.proposers.get(&round) {
            None => self.default_proposer,
            Some(round_proposer) => *round_proposer,
        };
        if self.timeout_rounds.contains(&round) {
            let mut sender = self.timeout_sender.clone();
            tokio::spawn(async move { sender.send(round).await });
        }
        proposer
    }
}
