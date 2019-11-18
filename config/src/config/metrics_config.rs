// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct MetricsConfig {
    pub dir: PathBuf,
    pub collection_interval_ms: u64,
}

impl Default for MetricsConfig {
    fn default() -> MetricsConfig {
        MetricsConfig {
            dir: PathBuf::from("metrics"),
            collection_interval_ms: 1000,
        }
    }
}
