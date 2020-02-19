// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use slog::FilterLevel;
use std::str::FromStr;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct LoggerConfig {
    // Use async logging
    pub is_async: bool,
    // chan_size of slog async drain for node logging.
    pub chan_size: usize,
    // The default logging level for slog.
    #[serde(serialize_with = "serialize_filter_level")]
    #[serde(deserialize_with = "deserialize_filter_level")]
    pub filter_level: FilterLevel,
}

impl Default for LoggerConfig {
    fn default() -> LoggerConfig {
        LoggerConfig {
            is_async: true,
            chan_size: 256,
            filter_level: FilterLevel::Info,
        }
    }
}

#[allow(clippy::trivially_copy_pass_by_ref)]
fn serialize_filter_level<S>(filter_level: &FilterLevel, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    filter_level.as_str().to_string().serialize(serializer)
}

fn deserialize_filter_level<'de, D>(deserializer: D) -> Result<FilterLevel, D::Error>
where
    D: Deserializer<'de>,
{
    let filter_level_string: String = Deserialize::deserialize(deserializer)?;
    FilterLevel::from_str(&filter_level_string).map_err(|_| {
        de::Error::unknown_variant(
            &filter_level_string,
            &[
                "OFF", "CRITICAL", "ERROR", "WARNING", "INFO", "DEBUG", "TRACE",
            ],
        )
    })
}
