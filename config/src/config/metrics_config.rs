// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct MetricsConfig {
    pub collection_interval_ms: u64,
    pub dir: PathBuf,
    pub enabled: bool,
    #[serde(skip)]
    data_dir: PathBuf,
}

impl Default for MetricsConfig {
    fn default() -> MetricsConfig {
        MetricsConfig {
            collection_interval_ms: 1000,
            data_dir: PathBuf::from("/opt/libra/data"),
            enabled: false,
            dir: PathBuf::from("metrics"),
        }
    }
}

impl MetricsConfig {
    pub fn dir(&self) -> PathBuf {
        if self.dir.is_relative() {
            self.data_dir.join(&self.dir)
        } else {
            self.dir.clone()
        }
    }

    pub fn set_data_dir(&mut self, data_dir: PathBuf) {
        self.data_dir = data_dir;
    }
}
