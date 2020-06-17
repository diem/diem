// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_metrics::{
    register_histogram, register_int_counter, register_int_counter_vec, register_int_gauge,
    DurationHistogram, Histogram, IntCounter, IntCounterVec, IntGauge,
};
use once_cell::sync::Lazy;

//////////////////////
// HEALTH COUNTERS
//////////////////////

/// Monitor counters, used by monitor! macro
pub static OP_COUNTERS: Lazy<libra_metrics::OpMetrics> =
    Lazy::new(|| libra_metrics::OpMetrics::new_and_registered("consensus"));

pub static ERROR_COUNT: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "libra_consensus_error_count",
        "Total number of errors in main loop"
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

//////////////////////
// SafetyRules COUNTERS
//////////////////////
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
//////////////////////
// PROPOSAL ELECTION
//////////////////////

/// Count of the block proposals sent by this validator since last restart
/// (both primary and secondary)
pub static PROPOSALS_COUNT: Lazy<IntCounter> = Lazy::new(|| {
    register_int_counter!("libra_consensus_proposals_count", "Count of the block proposals sent by this validator since last restart (both primary and secondary)").unwrap()
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
// RoundState COUNTERS
//////////////////////
/// This counter is set to the last round reported by the local round_state.
pub static CURRENT_ROUND: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "libra_consensus_current_round",
        "This counter is set to the last round reported by the local round_state."
    )
    .unwrap()
});

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
// SYNC MANAGER COUNTERS
////////////////////////
/// Counts the number of times the sync info message has been set since last restart.
pub static SYNC_INFO_MSGS_SENT_COUNT: Lazy<IntCounter> = Lazy::new(|| {
    register_int_counter!(
        "libra_consensus_sync_info_msg_sent_count",
        "Counts the number of times the sync info message has been set since last restart."
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
// TODO Consider reintroducing this counter
// pub static UNWRAPPED_PROPOSAL_SIZE_BYTES: Lazy<Histogram> = Lazy::new(|| {
//     register_histogram!(
//         "libra_consensus_unwrapped_proposal_size_bytes",
//         "Histogram of proposal size after LCS but before wrapping with GRPC and libra net."
//     )
//     .unwrap()
// });

/// Histogram for the number of txns per (committed) blocks.
pub static NUM_TXNS_PER_BLOCK: Lazy<Histogram> = Lazy::new(|| {
    register_histogram!(
        "libra_consensus_num_txns_per_block",
        "Histogram for the number of txns per (committed) blocks."
    )
    .unwrap()
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

/// Histogram of the time it requires to wait before inserting blocks into block store.
/// Measured as the block's timestamp minus local timestamp.
pub static WAIT_DURATION_S: Lazy<DurationHistogram> = Lazy::new(|| {
    DurationHistogram::new(register_histogram!("libra_consensus_wait_duration_s", "Histogram of the time it requires to wait before inserting blocks into block store. Measured as the block's timestamp minus the local timestamp.").unwrap())
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

/// Count of the pending outbound round timeouts
pub static PENDING_ROUND_TIMEOUTS: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "libra_consensus_pending_round_timeouts",
        "Count of the pending outbound round timeouts"
    )
    .unwrap()
});

/// Counter of pending network events to Consensus
pub static PENDING_CONSENSUS_NETWORK_EVENTS: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "libra_consensus_pending_network_events",
        "Counters(queued,dequeued,dropped) related to pending network notifications to Consensus",
        &["state"]
    )
    .unwrap()
});

/// Counters(queued,dequeued,dropped) related to consensus channel
pub static CONSENSUS_CHANNEL_MSGS: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "libra_consensus_channel_msgs_count",
        "Counters(queued,dequeued,dropped) related to consensus channel",
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
