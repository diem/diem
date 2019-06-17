// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    chained_bft::{
        common::{Payload, Round},
        liveness::proposer_election::{ProposalInfo, ProposerElection, ProposerInfo},
    },
    stream_utils::EventBasedActor,
};
use futures::{channel::mpsc, Future, FutureExt, SinkExt};
use logger::prelude::*;
use std::pin::Pin;

/// The rotating proposer maps a round to an author according to a round-robin rotation.
/// A fixed proposer strategy loses liveness when the fixed proposer is down. Rotating proposers
/// won't gather quorum certificates to machine loss/byzantine behavior on f/n rounds.
pub struct RotatingProposer<T, P> {
    // Ordering of proposers to rotate through (all honest replicas must agree on this)
    proposers: Vec<P>,
    // Number of contiguous rounds (i.e. round numbers increase by 1) a proposer is active
    // in a row
    contiguous_rounds: u32,
    // Output stream to send the chosen proposals
    winning_proposals: Option<mpsc::Sender<ProposalInfo<T, P>>>,
}

impl<T, P: ProposerInfo> RotatingProposer<T, P> {
    /// With only one proposer in the vector, it behaves the same as a fixed proposer strategy.
    pub fn new(proposers: Vec<P>, contiguous_rounds: u32) -> Self {
        Self {
            proposers,
            contiguous_rounds,
            winning_proposals: None,
        }
    }

    fn get_proposer(&self, round: Round) -> P {
        self.proposers
            [((round / u64::from(self.contiguous_rounds)) % self.proposers.len() as u64) as usize]
    }
}

impl<T: Payload, P: ProposerInfo> ProposerElection<T, P> for RotatingProposer<T, P> {
    fn is_valid_proposer(&self, author: P, round: Round) -> Option<P> {
        if self.get_proposer(round).get_author() == author.get_author() {
            Some(author)
        } else {
            None
        }
    }

    fn get_valid_proposers(&self, round: Round) -> Vec<P> {
        vec![self.get_proposer(round)]
    }
}

impl<T: Payload, P: ProposerInfo> EventBasedActor for RotatingProposer<T, P> {
    type InputEvent = ProposalInfo<T, P>;
    type OutputEvent = ProposalInfo<T, P>;

    fn init(
        &mut self,
        _: mpsc::Sender<Self::InputEvent>,
        output_stream_sender: mpsc::Sender<Self::OutputEvent>,
    ) {
        self.winning_proposals = Some(output_stream_sender);
    }

    fn process_event(&self, event: Self::InputEvent) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        let proposer = self.get_proposer(event.proposal.round());
        let mut sender = self.winning_proposals.as_ref().unwrap().clone();
        async move {
            if proposer.get_author() == event.proposer_info.get_author() {
                if let Err(e) = sender.send(event).await {
                    debug!("Error in sending the winning proposal: {:?}", e);
                }
            }
        }
            .boxed()
    }
}
