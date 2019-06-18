// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use lazy_static;
use metrics::OpMetrics;
use prometheus::{Histogram, IntCounter, IntGauge};

lazy_static::lazy_static! {
    pub static ref OP_COUNTERS: OpMetrics = OpMetrics::new_and_registered("consensus");
}

lazy_static::lazy_static! {
//////////////////////
// HEALTH COUNTERS
//////////////////////
/// This counter is set to the round of the highest committed block.
pub static ref LAST_COMMITTED_ROUND: IntGauge = OP_COUNTERS.gauge("last_committed_round");

/// The counter corresponds to the version of the last committed ledger info.
pub static ref LAST_COMMITTED_VERSION: IntGauge = OP_COUNTERS.gauge("last_committed_version");

/// This counter is set to the round of the highest voted block.
pub static ref LAST_VOTE_ROUND: IntGauge = OP_COUNTERS.gauge("last_vote_round");

/// This counter is set to the round of the preferred block (highest 2-chain head).
pub static ref PREFERRED_BLOCK_ROUND: IntGauge = OP_COUNTERS.gauge("preferred_block_round");

/// This counter is set to the last round reported by the local pacemaker.
pub static ref CURRENT_ROUND: IntGauge = OP_COUNTERS.gauge("current_round");

/// Count of the block proposals sent by this validator since last restart.
pub static ref PROPOSALS_COUNT: IntCounter = OP_COUNTERS.counter("proposals_count");

/// Count of the committed blocks since last restart.
pub static ref COMMITTED_BLOCKS_COUNT: IntCounter = OP_COUNTERS.counter("committed_blocks_count");

/// Count of the committed transactions since last restart.
pub static ref COMMITTED_TXNS_COUNT: IntCounter = OP_COUNTERS.counter("committed_txns_count");

/// Count of success txns in the blocks committed by this validator since last restart.
pub static ref SUCCESS_TXNS_COUNT: IntCounter = OP_COUNTERS.counter("success_txns_count");

/// Count of failed txns in the committed blocks since last restart.
/// FAILED_TXNS_COUNT + SUCCESS_TXN_COUNT == COMMITTED_TXNS_COUNT
pub static ref FAILED_TXNS_COUNT: IntCounter = OP_COUNTERS.counter("failed_txns_count");

//////////////////////
// PACEMAKER COUNTERS
//////////////////////
/// Count of the rounds that gathered QC since last restart.
pub static ref QC_ROUNDS_COUNT: IntCounter = OP_COUNTERS.counter("qc_rounds_count");

/// Count of the timeout rounds since last restart (close to 0 in happy path).
pub static ref TIMEOUT_ROUNDS_COUNT: IntCounter = OP_COUNTERS.counter("timeout_rounds_count");

/// Count the number of timeouts a node experienced since last restart (close to 0 in happy path).
/// This count is different from `TIMEOUT_ROUNDS_COUNT`, because not every time a node has
/// a timeout there is an ultimate decision to move to the next round (it might take multiple
/// timeouts to get the timeout certificate).
pub static ref TIMEOUT_COUNT: IntCounter = OP_COUNTERS.counter("timeout_count");

/// The timeout of the current round.
pub static ref ROUND_TIMEOUT_MS: IntGauge = OP_COUNTERS.gauge("round_timeout_ms");

////////////////////////
// SYNCMANAGER COUNTERS
////////////////////////
/// Count the number of times we invoked state synchronization since last restart.
pub static ref STATE_SYNC_COUNT: IntCounter = OP_COUNTERS.counter("state_sync_count");

/// Count the overall number of transactions state synchronizer has retrieved since last restart.
/// Large values mean that a node has been significantly behind and had to replay a lot of txns.
pub static ref STATE_SYNC_TXN_REPLAYED: IntCounter = OP_COUNTERS.counter("state_sync_txns_replayed");

/// Count the number of block retrieval requests issued since last restart.
pub static ref BLOCK_RETRIEVAL_COUNT: IntCounter = OP_COUNTERS.counter("block_retrieval_count");

/// Histogram of block retrieval duration.
pub static ref BLOCK_RETRIEVAL_DURATION_MS: Histogram = OP_COUNTERS.histogram("block_retrieval_duration_ms");

/// Histogram of state sync duration.
pub static ref STATE_SYNC_DURATION_MS: Histogram = OP_COUNTERS.histogram("state_sync_duration_ms");

//////////////////////
// RECONFIGURATION COUNTERS
//////////////////////
/// Current epoch num
pub static ref EPOCH_NUM: IntGauge = OP_COUNTERS.gauge("epoch_num");
/// The number of validators in the current epoch
pub static ref CURRENT_EPOCH_NUM_VALIDATORS: IntGauge = OP_COUNTERS.gauge("current_epoch_num_validators");
/// Quorum size in the current epoch
pub static ref CURRENT_EPOCH_QUORUM_SIZE: IntGauge = OP_COUNTERS.gauge("current_epoch_quorum_size");


//////////////////////
// BLOCK STORE COUNTERS
//////////////////////
/// Counter for the number of blocks in the block tree (including the root).
/// In a "happy path" with no collisions and timeouts, should be equal to 3 or 4.
pub static ref NUM_BLOCKS_IN_TREE: IntGauge = OP_COUNTERS.gauge("num_blocks_in_tree");

//////////////////////
// PERFORMANCE COUNTERS
//////////////////////
/// Histogram of execution time (ms) of non-empty blocks.
pub static ref BLOCK_EXECUTION_DURATION_MS: Histogram = OP_COUNTERS.histogram("block_execution_duration_ms");

/// Histogram of duration of a commit procedure (the time it takes for the execution / storage to
/// commit a block once we decide to do so).
pub static ref BLOCK_COMMIT_DURATION_MS: Histogram = OP_COUNTERS.histogram("block_commit_duration_ms");

/// Histogram for the number of txns per (committed) blocks.
pub static ref NUM_TXNS_PER_BLOCK: Histogram = OP_COUNTERS.histogram("num_txns_per_block");

/// Histogram of per-transaction execution time (ms) of non-empty blocks
/// (calculated as the overall execution time of a block divided by the number of transactions).
pub static ref TXN_EXECUTION_DURATION_MS: Histogram = OP_COUNTERS.histogram("txn_execution_duration_ms");

/// Histogram of execution time (ms) of empty blocks.
pub static ref EMPTY_BLOCK_EXECUTION_DURATION_MS: Histogram = OP_COUNTERS.histogram("empty_block_execution_duration_ms");

/// Histogram of the time it takes for a block to get committed.
/// Measured as the commit time minus block's timestamp.
pub static ref CREATION_TO_COMMIT_MS: Histogram = OP_COUNTERS.histogram("creation_to_commit_ms");

/// Duration between block generation time until the moment it gathers full QC
pub static ref CREATION_TO_QC_MS: Histogram = OP_COUNTERS.histogram("creation_to_qc_ms");

/// Duration between block generation time until the moment it is received and ready for execution.
pub static ref CREATION_TO_RECEIVAL_MS: Histogram = OP_COUNTERS.histogram("creation_to_receival_ms");

////////////////////////////////////
// PROPSOSAL/VOTE TIMESTAMP COUNTERS
////////////////////////////////////
/// Count of the proposals that passed the timestamp rules and did not have to wait
pub static ref PROPOSAL_NO_WAIT_REQUIRED_COUNT: IntCounter = OP_COUNTERS.counter("proposal_no_wait_required_count");

/// Count of the proposals where passing the timestamp rules required waiting
pub static ref PROPOSAL_WAIT_WAS_REQUIRED_COUNT: IntCounter = OP_COUNTERS.counter("proposal_wait_was_required_count");

/// Count of the proposals that were not made due to the waiting period exceeding the maximum allowed duration, breaking timestamp rules
pub static ref PROPOSAL_MAX_WAIT_EXCEEDED_COUNT: IntCounter = OP_COUNTERS.counter("proposal_max_wait_exceeded_count");

/// Count of the proposals that were not made due to waiting to ensure the current time exceeds min_duration_since_epoch failed, breaking timestamp rules
pub static ref PROPOSAL_WAIT_FAILED_COUNT: IntCounter = OP_COUNTERS.counter("proposal_wait_failed_count");

/// Histogram of time waited for successfully proposing a proposal (both those that waited and didn't wait) after following timestamp rules
pub static ref PROPOSAL_SUCCESS_WAIT_MS: Histogram = OP_COUNTERS.histogram("proposal_success_wait_ms");

/// Histogram of time waited for failing to propose a proposal (both those that waited and didn't wait) while trying to follow timestamp rules
pub static ref PROPOSAL_FAILURE_WAIT_MS: Histogram = OP_COUNTERS.histogram("proposal_failure_wait_ms");

/// Count of the votes that passed the timestamp rules and did not have to wait
pub static ref VOTE_NO_WAIT_REQUIRED_COUNT: IntCounter = OP_COUNTERS.counter("vote_no_wait_required_count");

/// Count of the votes where passing the timestamp rules required waiting
pub static ref VOTE_WAIT_WAS_REQUIRED_COUNT: IntCounter = OP_COUNTERS.counter("vote_wait_was_required_count");

/// Count of the votes that were not made due to the waiting period exceeding the maximum allowed duration, breaking timestamp rules
pub static ref VOTE_MAX_WAIT_EXCEEDED_COUNT: IntCounter = OP_COUNTERS.counter("vote_max_wait_exceeded_count");

/// Count of the votes that were not made due to waiting to ensure the current time exceeds min_duration_since_epoch failed, breaking timestamp rules
pub static ref VOTE_WAIT_FAILED_COUNT: IntCounter = OP_COUNTERS.counter("vote_wait_failed_count");

/// Histogram of time waited for successfully having the ability to vote (both those that waited and didn't wait) after following timestamp rules.
/// A success only means that a replica has an opportunity to vote.  It may not vote if it doesn't pass the voting rules.
pub static ref VOTE_SUCCESS_WAIT_MS: Histogram = OP_COUNTERS.histogram("vote_success_wait_ms");

/// Histogram of time waited for failing to have the ability to vote (both those that waited and didn't wait) while trying to follow timestamp rules
pub static ref VOTE_FAILURE_WAIT_MS: Histogram = OP_COUNTERS.histogram("vote_failure_wait_ms");

///////////////////
// CHANNEL COUNTERS
///////////////////
/// Count of the pending messages sent to itself in the channel
pub static ref PENDING_SELF_MESSAGES: IntGauge = OP_COUNTERS.gauge("pending_self_messages");

/// Count of the pending inbound proposals
pub static ref PENDING_PROPOSAL: IntGauge = OP_COUNTERS.gauge("pending_proposal");

/// Count of the pending inbound votes
pub static ref PENDING_VOTES: IntGauge = OP_COUNTERS.gauge("pending_votes");

/// Count of the pending inbound block requests
pub static ref PENDING_BLOCK_REQUESTS: IntGauge = OP_COUNTERS.gauge("pending_block_requests");

/// Count of the pending inbound chunk requests
pub static ref PENDING_CHUNK_REQUESTS: IntGauge = OP_COUNTERS.gauge("pending_chunk_requests");

/// Count of the pending inbound new round messages
pub static ref PENDING_NEW_ROUND_MESSAGES: IntGauge = OP_COUNTERS.gauge("pending_new_round_messages");

/// Count of the pending outbound pacemaker timeouts
pub static ref PENDING_PACEMAKER_TIMEOUTS: IntGauge = OP_COUNTERS.gauge("pending_pacemaker_timeouts");
}
