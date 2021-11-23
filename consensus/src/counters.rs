// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use diem_metrics::{
    register_histogram, register_histogram_vec, register_int_counter, register_int_counter_vec,
    register_int_gauge, DurationHistogram, Histogram, HistogramVec, IntCounter, IntCounterVec,
    IntGauge,
};
use once_cell::sync::Lazy;

//////////////////////
// HEALTH COUNTERS
//////////////////////

/// Monitor counters, used by monitor! macro
pub static OP_COUNTERS: Lazy<diem_metrics::OpMetrics> =
    Lazy::new(|| diem_metrics::OpMetrics::new_and_registered("consensus"));

pub static ERROR_COUNT: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "diem_consensus_error_count",
        "Total number of errors in main loop"
    )
    .unwrap()
});

/// This counter is set to the round of the highest committed block.
pub static LAST_COMMITTED_ROUND: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "diem_consensus_last_committed_round",
        "This counter is set to the round of the highest committed block."
    )
    .unwrap()
});

/// The counter corresponds to the version of the last committed ledger info.
pub static LAST_COMMITTED_VERSION: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "diem_consensus_last_committed_version",
        "The counter corresponds to the version of the last committed ledger info."
    )
    .unwrap()
});

/// Count of the committed blocks since last restart.
pub static COMMITTED_BLOCKS_COUNT: Lazy<IntCounter> = Lazy::new(|| {
    register_int_counter!(
        "diem_consensus_committed_blocks_count",
        "Count of the committed blocks since last restart."
    )
    .unwrap()
});

/// Count of the committed transactions since last restart.
pub static COMMITTED_TXNS_COUNT: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "diem_consensus_committed_txns_count",
        "Count of the transactions since last restart. state is success or failed",
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
    register_int_counter!("diem_consensus_proposals_count", "Count of the block proposals sent by this validator since last restart (both primary and secondary)").unwrap()
});

/// Count the number of times a validator voted for a nil block since last restart.
pub static VOTE_NIL_COUNT: Lazy<IntCounter> = Lazy::new(|| {
    register_int_counter!(
        "diem_consensus_vote_nil_count",
        "Count the number of times a validator voted for a nil block since last restart."
    )
    .unwrap()
});

/// Committed proposals from this validator when using LeaderReputation as the ProposerElection
pub static COMMITTED_PROPOSALS_IN_WINDOW: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "diem_committed_proposals_in_window",
        "Total number of this validator's committed proposals in the current reputation window"
    )
    .unwrap()
});

/// Committed votes from this validator when using LeaderReputation as the ProposerElection
pub static COMMITTED_VOTES_IN_WINDOW: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "diem_committed_votes_in_window",
        "Total number of this validator's committed votes in the current reputation window"
    )
    .unwrap()
});

//////////////////////
// RoundState COUNTERS
//////////////////////
/// This counter is set to the last round reported by the local round_state.
pub static CURRENT_ROUND: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "diem_consensus_current_round",
        "This counter is set to the last round reported by the local round_state."
    )
    .unwrap()
});

/// Count of the rounds that gathered QC since last restart.
pub static QC_ROUNDS_COUNT: Lazy<IntCounter> = Lazy::new(|| {
    register_int_counter!(
        "diem_consensus_qc_rounds_count",
        "Count of the rounds that gathered QC since last restart."
    )
    .unwrap()
});

/// Count of the timeout rounds since last restart (close to 0 in happy path).
pub static TIMEOUT_ROUNDS_COUNT: Lazy<IntCounter> = Lazy::new(|| {
    register_int_counter!(
        "diem_consensus_timeout_rounds_count",
        "Count of the timeout rounds since last restart (close to 0 in happy path)."
    )
    .unwrap()
});

/// Count the number of timeouts a node experienced since last restart (close to 0 in happy path).
/// This count is different from `TIMEOUT_ROUNDS_COUNT`, because not every time a node has
/// a timeout there is an ultimate decision to move to the next round (it might take multiple
/// timeouts to get the timeout certificate).
pub static TIMEOUT_COUNT: Lazy<IntCounter> = Lazy::new(|| {
    register_int_counter!("diem_consensus_timeout_count", "Count the number of timeouts a node experienced since last restart (close to 0 in happy path).").unwrap()
});

/// The timeout of the current round.
pub static ROUND_TIMEOUT_MS: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "diem_consensus_round_timeout_s",
        "The timeout of the current round."
    )
    .unwrap()
});

////////////////////////
// SYNC MANAGER COUNTERS
////////////////////////
/// Counts the number of times the sync info message has been set since last restart.
pub static SYNC_INFO_MSGS_SENT_COUNT: Lazy<IntCounter> = Lazy::new(|| {
    register_int_counter!(
        "diem_consensus_sync_info_msg_sent_count",
        "Counts the number of times the sync info message has been set since last restart."
    )
    .unwrap()
});

//////////////////////
// RECONFIGURATION COUNTERS
//////////////////////
/// Current epoch num
pub static EPOCH: Lazy<IntGauge> =
    Lazy::new(|| register_int_gauge!("diem_consensus_epoch", "Current epoch num").unwrap());

/// The number of validators in the current epoch
pub static CURRENT_EPOCH_VALIDATORS: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "diem_consensus_current_epoch_validators",
        "The number of validators in the current epoch"
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
        "diem_consensus_num_blocks_in_tree",
        "Counter for the number of blocks in the block tree (including the root)."
    )
    .unwrap()
});

//////////////////////
// PERFORMANCE COUNTERS
//////////////////////
// TODO Consider reintroducing this counter
// pub static UNWRAPPED_PROPOSAL_SIZE_BYTES: Lazy<Histogram> = Lazy::new(|| {
//     register_histogram!(
//         "diem_consensus_unwrapped_proposal_size_bytes",
//         "Histogram of proposal size after BCS but before wrapping with GRPC and diem net."
//     )
//     .unwrap()
// });

/// Histogram for the number of txns per (committed) blocks.
pub static NUM_TXNS_PER_BLOCK: Lazy<Histogram> = Lazy::new(|| {
    register_histogram!(
        "diem_consensus_num_txns_per_block",
        "Histogram for the number of txns per (committed) blocks."
    )
    .unwrap()
});

pub static BLOCK_TRACING: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        "diem_consensus_block_tracing",
        "Histogram for different stages of a block",
        &["stage"]
    )
    .unwrap()
});

/// Histogram of the time it requires to wait before inserting blocks into block store.
/// Measured as the block's timestamp minus local timestamp.
pub static WAIT_DURATION_S: Lazy<DurationHistogram> = Lazy::new(|| {
    DurationHistogram::new(register_histogram!("diem_consensus_wait_duration_s", "Histogram of the time it requires to wait before inserting blocks into block store. Measured as the block's timestamp minus the local timestamp.").unwrap())
});

///////////////////
// CHANNEL COUNTERS
///////////////////
/// Count of the pending messages sent to itself in the channel
pub static PENDING_SELF_MESSAGES: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "diem_consensus_pending_self_messages",
        "Count of the pending messages sent to itself in the channel"
    )
    .unwrap()
});

/// Count of the pending outbound round timeouts
pub static PENDING_ROUND_TIMEOUTS: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "diem_consensus_pending_round_timeouts",
        "Count of the pending outbound round timeouts"
    )
    .unwrap()
});

/// Counter of pending network events to Consensus
pub static PENDING_CONSENSUS_NETWORK_EVENTS: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "diem_consensus_pending_network_events",
        "Counters(queued,dequeued,dropped) related to pending network notifications to Consensus",
        &["state"]
    )
    .unwrap()
});

/// Count of the pending state sync notification.
pub static PENDING_STATE_SYNC_NOTIFICATION: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "diem_consensus_pending_state_sync_notification",
        "Count of the pending state sync notification"
    )
    .unwrap()
});

pub static BUFFER_MANAGER_MSGS: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "diem_consensus_buffer_manager_msgs_count",
        "Counters(queued,dequeued,dropped) related to pending commit votes",
        &["state"]
    )
    .unwrap()
});

/// Counters(queued,dequeued,dropped) related to consensus channel
pub static CONSENSUS_CHANNEL_MSGS: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "diem_consensus_channel_msgs_count",
        "Counters(queued,dequeued,dropped) related to consensus channel",
        &["state"]
    )
    .unwrap()
});

/// Counters(queued,dequeued,dropped) related to consensus channel
pub static ROUND_MANAGER_CHANNEL_MSGS: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "diem_consensus_round_manager_msgs_count",
        "Counters(queued,dequeued,dropped) related to consensus channel",
        &["state"]
    )
    .unwrap()
});

/// Counters(queued,dequeued,dropped) related to block retrieval channel
pub static BLOCK_RETRIEVAL_CHANNEL_MSGS: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "diem_consensus_block_retrieval_channel_msgs_count",
        "Counters(queued,dequeued,dropped) related to block retrieval channel",
        &["state"]
    )
    .unwrap()
});
