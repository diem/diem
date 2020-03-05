// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_logger::CHANNEL_SIZE;
use log::Level;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct LoggerConfig {
    // Use async logging
    pub is_async: bool,
    // channel size for the asychronous channel for node logging.
    pub chan_size: usize,
    // The default logging level for slog.
    pub level: Level,
}

impl Default for LoggerConfig {
    fn default() -> LoggerConfig {
        LoggerConfig {
            is_async: true,
            chan_size: CHANNEL_SIZE,
            level: Level::Info,
        }
    }
}
