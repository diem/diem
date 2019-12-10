// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct MetricsConfig {
    pub dir: PathBuf,
    pub collection_interval_ms: u64,
    #[serde(skip)]
    data_dir: PathBuf,
}

impl Default for MetricsConfig {
    fn default() -> MetricsConfig {
        MetricsConfig {
            dir: PathBuf::from("metrics"),
            collection_interval_ms: 1000,
            data_dir: PathBuf::from("/opt/libra/data"),
        }
    }
}

impl MetricsConfig {
    pub fn dir(&self) -> Option<PathBuf> {
        if self.dir.as_os_str().is_empty() {
            None
        } else if self.dir.is_relative() {
            Some(self.data_dir.join(&self.dir))
        } else {
            Some(self.dir.clone())
        }
    }

    pub fn set_data_dir(&mut self, data_dir: PathBuf) {
        self.data_dir = data_dir;
    }
}
