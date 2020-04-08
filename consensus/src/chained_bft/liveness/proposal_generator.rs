// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    chained_bft::block_storage::BlockReader,
    counters,
    state_replication::TxnManager,
    util::time_service::{wait_if_possible, TimeService, WaitingError, WaitingSuccess},
};
use anyhow::{bail, ensure, format_err, Context};
use consensus_types::{
    block::Block,
    block_data::BlockData,
    common::{Author, Payload, Round},
    quorum_cert::QuorumCert,
};
use libra_logger::prelude::*;
use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

#[cfg(test)]
#[path = "proposal_generator_test.rs"]
mod proposal_generator_test;

/// ProposalGenerator is responsible for generating the proposed block on demand: it's typically
/// used by a validator that believes it's a valid candidate for serving as a proposer at a given
/// round.
/// ProposalGenerator is the one choosing the branch to extend:
/// - round is given by the caller (typically determined by Pacemaker).
/// The transactions for the proposed block are delivered by TxnManager.
///
/// TxnManager should be aware of the pending transactions in the branch that it is extending,
/// such that it will filter them out to avoid transaction duplication.
pub struct ProposalGenerator<T> {
    // The account address of this validator
    author: Author,
    // Block store is queried both for finding the branch to extend and for generating the
    // proposed block.
    block_store: Arc<dyn BlockReader<Payload = T> + Send + Sync>,
    // Transaction manager is delivering the transactions.
    txn_manager: Box<dyn TxnManager<Payload = T>>,
    // Time service to generate block timestamps
    time_service: Arc<dyn TimeService>,
    // Max number of transactions to be added to a proposed block.
    max_block_size: u64,
    // Last round that a proposal was generated
    last_round_generated: Mutex<Round>,
}

impl<T: Payload> ProposalGenerator<T> {
    pub fn new(
        author: Author,
        block_store: Arc<dyn BlockReader<Payload = T> + Send + Sync>,
        txn_manager: Box<dyn TxnManager<Payload = T>>,
        time_service: Arc<dyn TimeService>,
        max_block_size: u64,
    ) -> Self {
        Self {
            author,
            block_store,
            txn_manager,
            time_service,
            max_block_size,
            last_round_generated: Mutex::new(0),
        }
    }

    pub fn author(&self) -> Author {
        self.author
    }

    /// Creates a NIL block proposal extending the highest certified block from the block store.
    pub fn generate_nil_block(&self, round: Round) -> anyhow::Result<Block<T>> {
        let hqc = self.ensure_highest_quorum_cert(round)?;
        Ok(Block::new_nil(round, hqc.as_ref().clone()))
    }

    /// Reconfiguration rule - we propose empty blocks with parents' timestamp
    /// after reconfiguration until it's committed
    pub fn generate_reconfig_empty_suffix(&self, round: Round) -> anyhow::Result<BlockData<T>> {
        let hqc = self.ensure_highest_quorum_cert(round)?;
        Ok(BlockData::new_proposal(
            T::default(),
            self.author,
            round,
            hqc.certified_block().timestamp_usecs(),
            hqc.as_ref().clone(),
        ))
    }

    /// The function generates a new proposal block: the returned future is fulfilled when the
    /// payload is delivered by the TxnManager implementation.  At most one proposal can be
    /// generated per round (no proposal equivocation allowed).
    /// Errors returned by the TxnManager implementation are propagated to the caller.
    /// The logic for choosing the branch to extend is as follows:
    /// 1. The function gets the highest head of a one-chain from block tree.
    /// The new proposal must extend hqc to ensure optimistic responsiveness.
    /// 2. The round is provided by the caller.
    /// 3. In case a given round is not greater than the calculated parent, return an OldRound
    /// error.
    pub fn generate_proposal(
        &mut self,
        round: Round,
        round_deadline: Instant,
    ) -> anyhow::Result<BlockData<T>> {
        {
            let mut last_round_generated = self.last_round_generated.lock().unwrap();
            if *last_round_generated < round {
                *last_round_generated = round;
            } else {
                bail!("Already proposed in the round {}", round);
            }
        }

        let hqc = self.ensure_highest_quorum_cert(round)?;

        if hqc.certified_block().has_reconfiguration() {
            return self.generate_reconfig_empty_suffix(round);
        }

        // One needs to hold the blocks with the references to the payloads while get_block is
        // being executed: pending blocks vector keeps all the pending ancestors of the extended
        // branch.
        let pending_blocks = self
            .block_store
            .path_from_root(hqc.certified_block().id())
            .ok_or_else(|| format_err!("HQC {} already pruned", hqc.certified_block().id()))?;

        // Exclude all the pending transactions: these are all the ancestors of
        // parent (including) up to the root (excluding).
        let exclude_payload = pending_blocks
            .iter()
            .flat_map(|block| block.payload())
            .collect();

        let block_timestamp = {
            match wait_if_possible(
                self.time_service.as_ref(),
                Duration::from_micros(hqc.certified_block().timestamp_usecs()),
                round_deadline,
            ) {
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
                            counters::PROPOSAL_SUCCESS_WAIT_S.observe_duration(wait_duration);
                            counters::PROPOSALS_GENERATED_COUNT
                                .with_label_values(&["wait_was_required"])
                                .inc();
                            current_duration_since_epoch
                        }
                        WaitingSuccess::NoWaitRequired {
                            current_duration_since_epoch,
                            ..
                        } => {
                            counters::PROPOSAL_SUCCESS_WAIT_S.observe_duration(Duration::new(0, 0));
                            counters::PROPOSALS_GENERATED_COUNT
                                .with_label_values(&["no_wait_required"])
                                .inc();
                            current_duration_since_epoch
                        }
                    }
                }
                Err(waiting_error) => {
                    match waiting_error {
                        WaitingError::MaxWaitExceeded => {
                            counters::PROPOSAL_FAILURE_WAIT_S.observe_duration(Duration::new(0, 0));
                            counters::PROPOSALS_GENERATED_COUNT
                                .with_label_values(&["max_wait_exceeded"])
                                .inc();
                            bail!(
                                "Waiting until parent block timestamp usecs {:?} would exceed the round duration {:?}, hence will not create a proposal for this round",
                                hqc.certified_block().timestamp_usecs(),
                                round_deadline);
                        }
                        WaitingError::WaitFailed {
                            current_duration_since_epoch,
                            wait_duration,
                        } => {
                            counters::PROPOSAL_FAILURE_WAIT_S.observe_duration(wait_duration);
                            counters::PROPOSALS_GENERATED_COUNT
                                .with_label_values(&["wait_failed"])
                                .inc();
                            bail!(
                                "Even after waiting for {:?}, parent block timestamp usecs {:?} >= current timestamp usecs {:?}, will not create a proposal for this round",
                                wait_duration,
                                hqc.certified_block().timestamp_usecs(),
                                current_duration_since_epoch);
                        }
                    };
                }
            }
        };

        let txns = self
            .txn_manager
            .pull_txns(self.max_block_size, exclude_payload)
            .context("Fail to retrieve txn")?;

        Ok(BlockData::new_proposal(
            txns,
            self.author,
            round,
            block_timestamp.as_micros() as u64,
            hqc.as_ref().clone(),
        ))
    }

    fn ensure_highest_quorum_cert(&self, round: Round) -> anyhow::Result<Arc<QuorumCert>> {
        let hqc = self.block_store.highest_quorum_cert();
        ensure!(
            hqc.certified_block().round() < round,
            "Given round {} is lower than hqc round {}",
            round,
            hqc.certified_block().round()
        );
        ensure!(
            !hqc.ends_epoch(),
            "The epoch has already ended,a proposal is not allowed to generated"
        );

        Ok(hqc)
    }
}
