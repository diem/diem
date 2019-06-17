// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::chained_bft::{common::Round, consensus_types::block::Block};

use crate::{
    chained_bft::{block_storage::BlockReader, common::Payload},
    counters,
    state_replication::TxnManager,
    time_service::{wait_if_possible, TimeService, WaitingError, WaitingSuccess},
};
use logger::prelude::*;
use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

#[cfg(test)]
#[path = "proposal_generator_test.rs"]
mod proposal_generator_test;

#[derive(Clone, Debug, PartialEq, Fail)]
/// ProposalGeneration logical errors (e.g., given round number is low).
pub enum ProposalGenerationError {
    /// The round of a certified block we'd like to extend is not lower than the provided round.
    #[fail(display = "GivenRoundTooLow")]
    GivenRoundTooLow(Round),
    #[fail(display = "TxnRetrievalError")]
    TxnRetrievalError,
    /// Local clock waiting completed, but the timestamp is still not greater than its parent
    #[fail(display = "CurrentTimeTooOld")]
    CurrentTimeTooOld,
    /// Local clock waiting would exceed round duration to allow the timestamp to be greater that
    /// its parent
    #[fail(display = "ExceedsMaxRoundDuration")]
    ExceedsMaxRoundDuration,
    /// Already proposed at this round (only a single proposal per round is allowed)
    #[fail(display = "CurrentTimeTooOld")]
    AlreadyProposed(Round),
}

/// ProposalGenerator is responsible for generating the proposed block on demand: it's typically
/// used by a validator that believes it's a valid candidate for serving as a proposer at a given
/// round.
/// ProposalGenerator is the one choosing the branch to extend:
/// - height is determined as parent.height + 1,
/// - round is given by the caller (typically determined by Pacemaker).
/// The transactions for the proposed block are delivered by TxnManager.
///
/// TxnManager should be aware of the pending transactions in the branch that it is extending,
/// such that it will filter them out to avoid transaction duplication.
pub struct ProposalGenerator<T> {
    // Block store is queried both for finding the branch to extend and for generating the
    // proposed block.
    block_store: Arc<dyn BlockReader<Payload = T> + Send + Sync>,
    // Transaction manager is delivering the transactions.
    txn_manager: Arc<dyn TxnManager<Payload = T>>,
    // Time service to generate block timestamps
    time_service: Arc<dyn TimeService>,
    // Max number of transactions to be added to a proposed block.
    max_block_size: u64,
    // Support increasing block timestamps
    enforce_increasing_timestamps: bool,
    // Last round that a proposal was generated
    last_round_generated: Mutex<Round>,
}

impl<T: Payload> ProposalGenerator<T> {
    pub fn new(
        block_store: Arc<dyn BlockReader<Payload = T> + Send + Sync>,
        txn_manager: Arc<TxnManager<Payload = T>>,
        time_service: Arc<dyn TimeService>,
        max_block_size: u64,
        enforce_increasing_timestamps: bool,
    ) -> Self {
        Self {
            block_store,
            txn_manager,
            time_service,
            max_block_size,
            enforce_increasing_timestamps,
            last_round_generated: Mutex::new(0),
        }
    }

    /// The function generates a new proposal block: the returned future is fulfilled when the
    /// payload is delivered by the TxnManager implementation.  At most one proposal can be
    /// generated per round (no proposal equivocation allowed).
    /// Errors returned by the TxnManager implementation are propagated to the caller.
    /// The logic for choosing the branch to extend is as follows:
    /// 1. The function gets the highest head of a one-chain from block tree.
    /// The new proposal must extend hqc_block to ensure optimistic responsiveness.
    /// 2. While the height is ultimately determined as the parent.height + 1, the round is provided
    /// by the caller.
    /// 3. In case a given round is not greater than the calculated parent, return an OldRound
    /// error.
    pub async fn generate_proposal(
        &self,
        round: Round,
        round_deadline: Instant,
    ) -> Result<Block<T>, ProposalGenerationError> {
        {
            let mut last_round_generated = self.last_round_generated.lock().unwrap();
            if *last_round_generated < round {
                *last_round_generated = round;
            } else {
                return Err(ProposalGenerationError::AlreadyProposed(round));
            }
        }

        let hqc_block = self.block_store.highest_certified_block();
        if hqc_block.round() >= round {
            // The given round is too low.
            return Err(ProposalGenerationError::GivenRoundTooLow(hqc_block.round()));
        }

        // One needs to hold the blocks with the references to the payloads while get_block is
        // being executed: pending blocks vector keeps all the pending ancestors of the extended
        // branch.
        let pending_blocks = match self.block_store.path_from_root(Arc::clone(&hqc_block)) {
            Some(res) => res,
            // In case the whole system moved forward between the check of a round and getting
            // path from root.
            None => {
                return Err(ProposalGenerationError::GivenRoundTooLow(hqc_block.round()));
            }
        };
        //let pending_blocks = self.get_pending_blocks(Arc::clone(&hqc_block));
        // Exclude all the pending transactions: these are all the ancestors of
        // parent (including) up to the root (excluding).
        let exclude_payload = pending_blocks
            .iter()
            .map(|block| block.get_payload())
            .collect();

        let block_timestamp = {
            if self.enforce_increasing_timestamps {
                match wait_if_possible(
                    self.time_service.as_ref(),
                    Duration::from_micros(hqc_block.timestamp_usecs()),
                    round_deadline,
                )
                .await
                {
                    Ok(waiting_success) => {
                        debug!(
                            "Success with {:?} for getting a valid timestamp for the next proposal",
                            waiting_success
                        );

                        match waiting_success {
                            WaitingSuccess::WaitWasRequired {
                                current_duration_since_epoch,
                                wait_duration,
                            } => {
                                counters::PROPOSAL_SUCCESS_WAIT_MS
                                    .observe(wait_duration.as_millis() as f64);
                                counters::PROPOSAL_WAIT_WAS_REQUIRED_COUNT.inc();
                                current_duration_since_epoch
                            }
                            WaitingSuccess::NoWaitRequired {
                                current_duration_since_epoch,
                                ..
                            } => {
                                counters::PROPOSAL_SUCCESS_WAIT_MS.observe(0.0);
                                counters::PROPOSAL_NO_WAIT_REQUIRED_COUNT.inc();
                                current_duration_since_epoch
                            }
                        }
                    }
                    Err(waiting_error) => {
                        match waiting_error {
                            WaitingError::MaxWaitExceeded => {
                                error!(
                                    "Waiting until parent block timestamp usecs {:?} would exceed the round duration {:?}, hence will not create a proposal for this round",
                                    hqc_block.timestamp_usecs(),
                                    round_deadline);
                                counters::PROPOSAL_FAILURE_WAIT_MS.observe(0.0);
                                counters::PROPOSAL_MAX_WAIT_EXCEEDED_COUNT.inc();
                                return Err(ProposalGenerationError::ExceedsMaxRoundDuration);
                            }
                            WaitingError::WaitFailed {
                                current_duration_since_epoch,
                                wait_duration,
                            } => {
                                error!(
                                    "Even after waiting for {:?}, parent block timestamp usecs {:?} >= current timestamp usecs {:?}, will not create a proposal for this round",
                                    wait_duration,
                                    hqc_block.timestamp_usecs(),
                                    current_duration_since_epoch);
                                counters::PROPOSAL_FAILURE_WAIT_MS
                                    .observe(wait_duration.as_millis() as f64);
                                counters::PROPOSAL_WAIT_FAILED_COUNT.inc();
                                return Err(ProposalGenerationError::CurrentTimeTooOld);
                            }
                        };
                    }
                }
            } else {
                self.time_service.get_current_timestamp()
            }
        };

        let block_store = Arc::clone(&self.block_store);
        match self
            .txn_manager
            .pull_txns(self.max_block_size, exclude_payload)
            .await
        {
            Ok(txns) => Ok(block_store.create_block(
                hqc_block,
                txns,
                round,
                block_timestamp.as_micros() as u64,
            )),
            Err(_) => Err(ProposalGenerationError::TxnRetrievalError),
        }
    }
}
