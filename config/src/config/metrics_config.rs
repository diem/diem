// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::config::BaseConfig;
use failure::Result;
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, sync::Arc};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct MetricsConfig {
    pub dir: PathBuf,
    pub collection_interval_ms: u64,
    #[serde(skip)]
    pub base: Arc<BaseConfig>,
}

impl Default for MetricsConfig {
    fn default() -> MetricsConfig {
        MetricsConfig {
            dir: PathBuf::from("metrics"),
            collection_interval_ms: 1000,
            base: Arc::new(BaseConfig::default()),
        }
    }
}

impl MetricsConfig {
    pub fn load(&mut self, base: Arc<BaseConfig>) -> Result<()> {
        self.base = base;
        Ok(())
    }

    pub fn dir(&self) -> Option<PathBuf> {
        if self.dir.as_os_str().is_empty() {
            None
        } else {
            Some(self.base.full_path(&self.dir))
        }
    }
}
