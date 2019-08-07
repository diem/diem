// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use lazy_static;
use metrics::OpMetrics;
use prometheus::IntCounter;

lazy_static::lazy_static! {
    pub static ref OP_COUNTERS: OpMetrics = OpMetrics::new_and_registered("state_sync");
}

lazy_static::lazy_static! {
/// Count the overall number of transactions state synchronizer has retrieved since last restart.
/// Large values mean that a node has been significantly behind and had to replay a lot of txns.
pub static ref STATE_SYNC_TXN_REPLAYED: IntCounter = OP_COUNTERS.counter("state_sync_txns_replayed");

}
