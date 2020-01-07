// Copyright 2018 The Grin Developers
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Public Types used for cuckoo-miner module

use plugin::SolverParams;
use std::path::PathBuf;
use std::{fmt, io};
use {CuckooMinerError, PluginLibrary};

pub static SO_SUFFIX: &str = ".cuckooplugin";

/// CuckooMinerPlugin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    /// The display name of the plugin to load
    pub name: String,

    /// The path to the file
    pub file: String,

    /// device params
    pub params: SolverParams,
}

impl PluginConfig {
    /// create new!
    pub fn new(mut plugin_dir: PathBuf, name: &str) -> Result<PluginConfig, CuckooMinerError> {
        plugin_dir.push(format!("{}{}", name, SO_SUFFIX).as_str());
        let plugin_file_str = plugin_dir.to_str().ok_or_else(|| {
            CuckooMinerError::PluginNotFoundError(
                "Invalid plugin path. Paths must be valid unicode".to_owned(),
            )
        })?;

        PluginLibrary::new(plugin_file_str).map(|plugin_library| {
            let params = plugin_library.get_default_params();
            plugin_library.unload();
            PluginConfig {
                name: name.to_owned(),
                file: plugin_file_str.to_owned(),
                params,
            }
        })
    }
}

/// Error type wrapping config errors.
#[derive(Debug)]
#[allow(dead_code)]
pub enum ConfigError {
    /// Error with parsing of config file
    ParseError(String, String),

    /// Error with fileIO while reading config file
    FileIOError(String, String),

    /// No file found
    FileNotFoundError(String),

    /// Error serializing config values
    SerializationError(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ConfigError::ParseError(ref file_name, ref message) => write!(
                f,
                "Error parsing configuration file at {} - {}",
                file_name, message
            ),
            ConfigError::FileIOError(ref file_name, ref message) => {
                write!(f, "{} {}", message, file_name)
            }
            ConfigError::FileNotFoundError(ref file_name) => {
                write!(f, "Configuration file not found: {}", file_name)
            }
            ConfigError::SerializationError(ref message) => {
                write!(f, "Error serializing configuration: {}", message)
            }
        }
    }
}

impl From<io::Error> for ConfigError {
    fn from(error: io::Error) -> ConfigError {
        ConfigError::FileIOError(
            String::from(""),
            String::from(format!("Error loading config file: {}", error)),
        )
    }
}
