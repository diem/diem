// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use once_cell::sync::Lazy;
use prometheus::{HistogramVec, IntCounterVec, IntGauge};

// Number of transactions that admission control proxied up to upstream peer
//
// The 'success' of a transaction proxy is defined from a networking perspective, where the
// transaction is submitted to the upstream peer, receives a valid SubmitTransactionResponse,
// and callback to the downstream peer that initially sent this transaction submission request
// is also successful (i.e. no timeout)
//
// Thus, a transaction submission response from the upstream peer that states that
// the transaction was rejected by the upstream peer, would still be considered a
// successful transaction proxy
pub static TRANSACTION_PROXY: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "libra_admission_control_transaction_proxy_count",
        "Number of transactions a full node proxied up to upstream peer",
        &[
            // role of the node that sent the txn submission request to this node: client or FN
            "sender_role",
            // result of the txn proxy: success, failure
            "state",
        ]
    )
    .unwrap()
});

// Number of transactions that admission control submits to mempool
pub static TRANSACTION_SUBMISSION: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "libra_admission_control_transaction_submission_count",
        "Number of transactions AC in validator node tries to submit to mempool",
        &[
            // state of the transaction: accepted, rejected
            "state",
            // if the txn submission failed, the cause of rejection. "" for successful txn submission
            "rejection_cause",
        ]
    )
    .unwrap()
});

// Number of timeouts that happen when admission control handles transaction submission
// requests. Timeout can happen while waiting for a response from upstream peer
// or callback to the node that sent the submission request to this node
pub static TIMEOUT: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "libra_admission_control_timeout_count",
        "Number of timeouts that happen when AC handles transaction submissions",
        &[
            // role of the node that sent the txn submission request to this node: client or FN
            "sender_role",
            // type of timeout: "timeout", "callback_timeout"
            "state",
        ]
    )
    .unwrap()
});

// The time it takes for admission control to handle a transaction submission request, from when
// it sends the transaction to mempool or upstream peer (depending on its role), to when the
// callback to the downstream peer finishes
pub static TRANSACTION_LATENCY: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
    "libra_admission_control_transaction_submission_latency_s",
    "Histogram of time it takes for admission control to handle a transaction submission request",
    &[
        // result of the transaction handling: success, failure
        "state",
    ]
)
    .unwrap()
});

pub static UPSTREAM_PEERS: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "libra_admission_control_upstream_peer_count",
        "Number of upstream peers a node can proxy transactions to"
    )
    .unwrap()
});
