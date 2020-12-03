// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use diem_logger::{Level, CHANNEL_SIZE};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct LoggerConfig {
    // channel size for the asychronous channel for node logging.
    pub chan_size: usize,
    // Use async logging
    pub is_async: bool,
    // The default logging level for slog.
    pub level: Level,
}

impl Default for LoggerConfig {
    fn default() -> LoggerConfig {
        LoggerConfig {
            chan_size: CHANNEL_SIZE,
            is_async: true,
            level: Level::Info,
        }
    }
}
