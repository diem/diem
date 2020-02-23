// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_metrics::DurationHistogram;
use once_cell::sync::Lazy;
use prometheus::{Histogram, IntCounter, IntCounterVec, IntGauge};

//////////////////////
// HEALTH COUNTERS
//////////////////////

/// Counter of pending network events to Consensus
pub static PENDING_CONSENSUS_NETWORK_EVENTS: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "libra_consensus_pending_network_events",
        "Counters(queued,dequeued,dropped) related to pending network notifications to Consensus",
        &["state"]
    )
    .unwrap()
});

/// This counter is set to the round of the highest committed block.
pub static LAST_COMMITTED_ROUND: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "libra_consensus_last_committed_round",
        "This counter is set to the round of the highest committed block."
    )
    .unwrap()
});

/// The counter corresponds to the version of the last committed ledger info.
pub static LAST_COMMITTED_VERSION: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "libra_consensus_last_committed_version",
        "The counter corresponds to the version of the last committed ledger info."
    )
    .unwrap()
});

/// This counter is set to the round of the highest voted block.
pub static LAST_VOTE_ROUND: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "libra_consensus_last_vote_round",
        "This counter is set to the round of the highest voted block."
    )
    .unwrap()
});

/// This counter is set to the round of the preferred block (highest 2-chain head).
pub static PREFERRED_BLOCK_ROUND: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "libra_consensus_preferred_block_round",
        "This counter is set to the round of the preferred block (highest 2-chain head)."
    )
    .unwrap()
});

/// This counter is set to the last round reported by the local pacemaker.
pub static CURRENT_ROUND: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "libra_consensus_current_round",
        "This counter is set to the last round reported by the local pacemaker."
    )
    .unwrap()
});

/// Count of the committed blocks since last restart.
pub static COMMITTED_BLOCKS_COUNT: Lazy<IntCounter> = Lazy::new(|| {
    register_int_counter!(
        "libra_consensus_committed_blocks_count",
        "Count of the committed blocks since last restart."
    )
    .unwrap()
});

/// Count of the committed transactions since last restart.
pub static COMMITTED_TXNS_COUNT: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "libra_consensus_committed_txns_count",
        "Count of the transactions since last restart. state is success or failed",
        &["state"]
    )
    .unwrap()
});

/// Histogram of idle time of spent in event processing loop
pub static STARTUP_SYNC_LOOP_IDLE_DURATION_S: Lazy<DurationHistogram> = Lazy::new(|| {
    DurationHistogram::new(
        register_histogram!(
            "libra_consensus_startup_sync_loop_idle_duration_s",
            "Histogram of idle time of spent in startup sync loop"
        )
        .unwrap(),
    )
});

/// Histogram of idle time of spent in event processing loop
pub static STARTUP_SYNC_LOOP_BUSY_DURATION_S: Lazy<DurationHistogram> = Lazy::new(|| {
    DurationHistogram::new(
        register_histogram!(
            "libra_consensus_startup_sync_loop_busy_duration_s",
            "Histogram of busy time of spent in startup sync loop"
        )
        .unwrap(),
    )
});

/// Histogram of idle time of spent in event processing loop
pub static EVENT_PROCESSING_LOOP_IDLE_DURATION_S: Lazy<DurationHistogram> = Lazy::new(|| {
    DurationHistogram::new(
        register_histogram!(
            "libra_consensus_event_processing_loop_idle_duration_s",
            "Histogram of idle time of spent in event processing loop"
        )
        .unwrap(),
    )
});

/// Histogram of busy time of spent in event processing loop
pub static EVENT_PROCESSING_LOOP_BUSY_DURATION_S: Lazy<DurationHistogram> = Lazy::new(|| {
    DurationHistogram::new(
        register_histogram!(
            "libra_consensus_event_processing_loop_busy_duration_s",
            "Histogram of busy time of spent in event processing loop"
        )
        .unwrap(),
    )
});

/// Counters(queued,dequeued,dropped) related to proposals channel
pub static PROPOSAL_CHANNEL_MSGS: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "libra_consensus_proposal_channel_msgs_count",
        "Counters(queued,dequeued,dropped) related to proposals channel",
        &["state"]
    )
    .unwrap()
});

/// Counters(queued,dequeued,dropped) related to votes channel
pub static VOTES_CHANNEL_MSGS: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "libra_consensus_votes_channel_msgs_count",
        "Counters(queued,dequeued,dropped) related to votes channel",
        &["state"]
    )
    .unwrap()
});

/// Counters(queued,dequeued,dropped) related to block retrieval channel
pub static BLOCK_RETRIEVAL_CHANNEL_MSGS: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "libra_consensus_block_retrieval_channel_msgs_count",
        "Counters(queued,dequeued,dropped) related to block retrieval channel",
        &["state"]
    )
    .unwrap()
});

/// Counters(queued,dequeued,dropped) related to sync info channel
pub static SYNC_INFO_CHANNEL_MSGS: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "libra_consensus_sync_info_dropped_channel_msgs_count",
        "Counters(queued,dequeued,dropped) related to sync info channel",
        &["state"]
    )
    .unwrap()
});

/// Counters(queued,dequeued,dropped) related to epoch change channel
pub static EPOCH_CHANGE_CHANNEL_MSGS: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "libra_consensus_epoch_change_dropped_channel_msgs_count",
        "Counters(queued,dequeued,dropped) related to epoch change channel",
        &["state"]
    )
    .unwrap()
});

//////////////////////
// PROPOSAL ELECTION
//////////////////////

/// Count of the block proposals sent by this validator since last restart
/// (both primary and secondary)
pub static PROPOSALS_COUNT: Lazy<IntCounter> = Lazy::new(|| {
    register_int_counter!("libra_consensus_proposals_count", "Count of the block proposals sent by this validator since last restart (both primary and secondary)").unwrap()
});

/// Count the number of times a validator voted for secondary proposals (upon timeout) since
/// last restart.
pub static VOTE_SECONDARY_PROPOSAL_COUNT: Lazy<IntCounter> = Lazy::new(|| {
    register_int_counter!("libra_consensus_vote_secondary_proposal_count", "Count the number of times a validator voted for secondary proposals (upon timeout) since last restart.").unwrap()
});

/// Count the number of times a validator voted for a nil block since last restart.
pub static VOTE_NIL_COUNT: Lazy<IntCounter> = Lazy::new(|| {
    register_int_counter!(
        "libra_consensus_vote_nil_count",
        "Count the number of times a validator voted for a nil block since last restart."
    )
    .unwrap()
});

//////////////////////
// PACEMAKER COUNTERS
//////////////////////
/// Count of the rounds that gathered QC since last restart.
pub static QC_ROUNDS_COUNT: Lazy<IntCounter> = Lazy::new(|| {
    register_int_counter!(
        "libra_consensus_qc_rounds_count",
        "Count of the rounds that gathered QC since last restart."
    )
    .unwrap()
});

/// Count of the timeout rounds since last restart (close to 0 in happy path).
pub static TIMEOUT_ROUNDS_COUNT: Lazy<IntCounter> = Lazy::new(|| {
    register_int_counter!(
        "libra_consensus_timeout_rounds_count",
        "Count of the timeout rounds since last restart (close to 0 in happy path)."
    )
    .unwrap()
});

/// Count the number of timeouts a node experienced since last restart (close to 0 in happy path).
/// This count is different from `TIMEOUT_ROUNDS_COUNT`, because not every time a node has
/// a timeout there is an ultimate decision to move to the next round (it might take multiple
/// timeouts to get the timeout certificate).
pub static TIMEOUT_COUNT: Lazy<IntCounter> = Lazy::new(|| {
    register_int_counter!("libra_consensus_timeout_count", "Count the number of timeouts a node experienced since last restart (close to 0 in happy path).").unwrap()
});

/// The timeout of the current round.
pub static ROUND_TIMEOUT_MS: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "libra_consensus_round_timeout_s",
        "The timeout of the current round."
    )
    .unwrap()
});

////////////////////////
// SYNCMANAGER COUNTERS
////////////////////////
/// Count the number of times we invoked state synchronization since last restart.
pub static STATE_SYNC_COUNT: Lazy<IntCounter> = Lazy::new(|| {
    register_int_counter!(
        "libra_consensus_state_sync_count",
        "Count the number of times we invoked state synchronization since last restart."
    )
    .unwrap()
});

/// Count the number of block retrieval requests issued since last restart.
pub static BLOCK_RETRIEVAL_COUNT: Lazy<IntCounter> = Lazy::new(|| {
    register_int_counter!(
        "libra_consensus_block_retrieval_count",
        "Count the number of block retrieval requests issued since last restart."
    )
    .unwrap()
});

/// Histogram of block retrieval duration.
pub static BLOCK_RETRIEVAL_DURATION_S: Lazy<DurationHistogram> = Lazy::new(|| {
    DurationHistogram::new(
        register_histogram!(
            "libra_consensus_block_retrieval_duration_s",
            "Histogram of block retrieval duration."
        )
        .unwrap(),
    )
});

/// Histogram of state sync duration.
pub static STATE_SYNC_DURATION_S: Lazy<DurationHistogram> = Lazy::new(|| {
    DurationHistogram::new(
        register_histogram!(
            "libra_consensus_state_sync_duration_s",
            "Histogram of state sync duration."
        )
        .unwrap(),
    )
});

/// Counts the number of times the sync info message has been set since last restart.
pub static SYNC_INFO_MSGS_SENT_COUNT: Lazy<IntCounter> = Lazy::new(|| {
    register_int_counter!(
        "libra_consensus_sync_info_msg_sent_count",
        "Counts the number of times the sync info message has been set since last restart."
    )
    .unwrap()
});

/// Counts the number of times the sync info message has been received since last restart.
pub static SYNC_INFO_MSGS_RECEIVED_COUNT: Lazy<IntCounter> = Lazy::new(|| {
    register_int_counter!(
        "libra_consensus_sync_info_msg_received_count",
        "Counts the number of times the sync info message has been received since last restart."
    )
    .unwrap()
});

//////////////////////
// RECONFIGURATION COUNTERS
//////////////////////
/// Current epoch num
pub static EPOCH: Lazy<IntGauge> =
    Lazy::new(|| register_int_gauge!("libra_consensus_epoch", "Current epoch num").unwrap());
/// The number of validators in the current epoch
pub static CURRENT_EPOCH_VALIDATORS: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "libra_consensus_current_epoch_validators",
        "The number of validators in the current epoch"
    )
    .unwrap()
});
/// Quorum size in the current epoch
pub static CURRENT_EPOCH_QUORUM_SIZE: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "libra_consensus_current_epoch_quorum_size",
        "Quorum size in the current epoch"
    )
    .unwrap()
});

//////////////////////
// BLOCK STORE COUNTERS
//////////////////////
/// Counter for the number of blocks in the block tree (including the root).
/// In a "happy path" with no collisions and timeouts, should be equal to 3 or 4.
pub static NUM_BLOCKS_IN_TREE: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "libra_consensus_num_blocks_in_tree",
        "Counter for the number of blocks in the block tree (including the root)."
    )
    .unwrap()
});

//////////////////////
// PERFORMANCE COUNTERS
//////////////////////
/// Histogram of execution time of non-empty blocks.
pub static BLOCK_EXECUTION_DURATION_S: Lazy<DurationHistogram> = Lazy::new(|| {
    DurationHistogram::new(
        register_histogram!(
            "libra_consensus_block_execution_duration_s",
            "Histogram of execution time of non-empty blocks."
        )
        .unwrap(),
    )
});

pub static UNWRAPPED_PROPOSAL_SIZE_BYTES: Lazy<Histogram> = Lazy::new(|| {
    register_histogram!(
        "libra_consensus_unwrapped_proposal_size_bytes",
        "Histogram of proposal size after LCS but before wrapping with GRPC and libra net."
    )
    .unwrap()
});

/// Histogram of duration of a commit procedure (the time it takes for the execution / storage to
/// commit a block once we decide to do so).
pub static BLOCK_COMMIT_DURATION_S: Lazy<DurationHistogram> = Lazy::new(|| {
    DurationHistogram::new(register_histogram!("libra_consensus_block_commit_duration_s", "Histogram of duration of a commit procedure (the time it takes for the execution / storage to commit a block once we decide to do so).").unwrap())
});

/// Histogram for the number of txns per (committed) blocks.
pub static NUM_TXNS_PER_BLOCK: Lazy<Histogram> = Lazy::new(|| {
    register_histogram!(
        "libra_consensus_num_txns_per_block",
        "Histogram for the number of txns per (committed) blocks."
    )
    .unwrap()
});

/// Histogram of per-transaction execution time of non-empty blocks
/// (calculated as the overall execution time of a block divided by the number of transactions).
pub static TXN_EXECUTION_DURATION_S: Lazy<DurationHistogram> = Lazy::new(|| {
    DurationHistogram::new(register_histogram!("libra_consensus_txn_execution_duration_s", "Histogram of per-transaction execution time of non-empty blocks (calculated as the overall execution time of a block divided by the number of transactions).").unwrap())
});

/// Histogram of the time it takes for a block to get committed.
/// Measured as the commit time minus block's timestamp.
pub static CREATION_TO_COMMIT_S: Lazy<DurationHistogram> = Lazy::new(|| {
    DurationHistogram::new(register_histogram!("libra_consensus_creation_to_commit_s", "Histogram of the time it takes for a block to get committed. Measured as the commit time minus block's timestamp.").unwrap())
});

/// Duration between block generation time until the moment it gathers full QC
pub static CREATION_TO_QC_S: Lazy<DurationHistogram> = Lazy::new(|| {
    DurationHistogram::new(
        register_histogram!(
            "libra_consensus_creation_to_qc_s",
            "Duration between block generation time until the moment it gathers full QC"
        )
        .unwrap(),
    )
});

/// Duration between block generation time until the moment it is received and ready for execution.
pub static CREATION_TO_RECEIVAL_S: Lazy<DurationHistogram> = Lazy::new(|| {
    DurationHistogram::new(register_histogram!("libra_consensus_creation_to_receival_s", "Duration between block generation time until the moment it is received and ready for execution.").unwrap())
});

////////////////////////////////////
// PROPSOSAL/VOTE TIMESTAMP COUNTERS
////////////////////////////////////

/// Total count of the proposals generated. state can be:
/// no_wait_required: Count of the proposals that passed the timestamp rules and did not have to wait
/// wait_was_required: Count of the proposals where passing the timestamp rules required waiting
/// max_wait_exceeded: Count of the proposals that were not made due to the waiting period exceeding the maximum allowed duration, breaking timestamp rules
/// wait_failed: Count of the proposals that were not made due to waiting to ensure the current time exceeds min_duration_since_epoch failed, breaking timestamp rules
pub static PROPOSALS_GENERATED_COUNT: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "libra_consensus_proposals_generated_count",
        "Count of all the proposals generated",
        &["state"]
    )
    .unwrap()
});

/// Histogram of time waited for successfully proposing a proposal (both those that waited and didn't wait) after following timestamp rules
pub static PROPOSAL_SUCCESS_WAIT_S: Lazy<DurationHistogram> = Lazy::new(|| {
    DurationHistogram::new(register_histogram!("libra_consensus_proposal_success_wait_s", "Histogram of time waited for successfully proposing a proposal (both those that waited and didn't wait) after following timestamp rules").unwrap())
});

/// Histogram of time waited for failing to propose a proposal (both those that waited and didn't wait) while trying to follow timestamp rules
pub static PROPOSAL_FAILURE_WAIT_S: Lazy<DurationHistogram> = Lazy::new(|| {
    DurationHistogram::new(register_histogram!("libra_consensus_proposal_failure_wait_s", "Histogram of time waited for failing to propose a proposal (both those that waited and didn't wait) while trying to follow timestamp rules").unwrap())
});

/// Total count of the votes. state can be:
/// no_wait_required: Count of the votes that passed the timestamp rules and did not have to wait
/// wait_was_required: Count of the votes where passing the timestamp rules required waiting
/// max_wait_exceeded: Count of the votes that were not made due to the waiting period exceeding the maximum allowed duration, breaking timestamp rules
/// wait_failed: Count of the votes that were not made due to waiting to ensure the current time exceeds min_duration_since_epoch failed, breaking timestamp rules
pub static VOTES_COUNT: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "libra_consensus_votes_count",
        "Count of all the votes",
        &["state"]
    )
    .unwrap()
});

/// Histogram of time waited for successfully having the ability to vote (both those that waited and didn't wait) after following timestamp rules.
/// A success only means that a replica has an opportunity to vote.  It may not vote if it doesn't pass the voting rules.
pub static VOTE_SUCCESS_WAIT_S: Lazy<DurationHistogram> = Lazy::new(|| {
    DurationHistogram::new(register_histogram!("libra_consensus_vote_success_wait_s", "Histogram of time waited for successfully having the ability to vote (both those that waited and didn't wait) after following timestamp rules.").unwrap())
});

/// Histogram of time waited for failing to have the ability to vote (both those that waited and didn't wait) while trying to follow timestamp rules
pub static VOTE_FAILURE_WAIT_S: Lazy<DurationHistogram> = Lazy::new(|| {
    DurationHistogram::new(register_histogram!("libra_consensus_vote_success_wait_s", "Histogram of time waited for failing to have the ability to vote (both those that waited and didn't wait) while trying to follow timestamp rules").unwrap())
});

///////////////////
// CHANNEL COUNTERS
///////////////////
/// Count of the pending messages sent to itself in the channel
pub static PENDING_SELF_MESSAGES: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "libra_consensus_pending_self_messages",
        "Count of the pending messages sent to itself in the channel"
    )
    .unwrap()
});

/// Count of the pending outbound pacemaker timeouts
pub static PENDING_PACEMAKER_TIMEOUTS: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "libra_consensus_pending_pacemaker_timeouts",
        "Count of the pending outbound pacemaker timeouts"
    )
    .unwrap()
});
