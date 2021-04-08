// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::counters;
use diem_infallible::duration_since_epoch;
use std::time::Duration;

pub struct BlockStage;

impl BlockStage {
    pub const SIGNED: &'static str = "signed";
    pub const RECEIVED: &'static str = "received";
    pub const SYNCED: &'static str = "synced";
    pub const EXECUTED: &'static str = "executed";
    pub const VOTED: &'static str = "voted";
    pub const QC_AGGREGATED: &'static str = "qc_aggregated";
    pub const QC_ADDED: &'static str = "qc_added";
    pub const ORDERED: &'static str = "ordered";
    pub const COMMITTED: &'static str = "committed";
}

/// Record the time during each stage of a block.
pub fn observe_block(timestamp: u64, stage: &'static str) {
    if let Some(t) = duration_since_epoch().checked_sub(Duration::from_micros(timestamp)) {
        counters::BLOCK_TRACING
            .with_label_values(&[stage])
            .observe(t.as_secs_f64());
    }
}
