// Copyright 2017 The Grin Developers
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

//! Common error type used by all cuckoo-miner modules, as well as any exernal
//! consumers of the cuckoo-miner crate.

use std::io;
use std::string;

/// #Description
///
/// Top level enum for all errors that the cuckoo-miner crate can return.
///

#[derive(Debug)]
pub enum CuckooMinerError {
	/// Occurs when trying to call a plugin function when a
	/// mining plugin is not loaded.
	PluginNotLoadedError(String),

	/// Occurs when trying to load plugin function that doesn't exist
	PluginSymbolNotFoundError(String),

	/// Occurs when attempting to load a plugin that doesn't exist
	PluginNotFoundError(String),

	/// Occurs when trying to load a plugin directory that doesn't
	/// contain any plugins
	NoPluginsFoundError(String),

	/// Unexpected return code from a plugin
	UnexpectedResultError(u32),

	/// Error setting a parameter
	ParameterError(String),

	/// IO Error
	PluginIOError(String),

	/// Plugin processing can't start
	PluginProcessingError(String),

	/// Error getting stats or stats not implemented
	StatsError(String),
}

impl From<io::Error> for CuckooMinerError {
	fn from(error: io::Error) -> Self {
		CuckooMinerError::PluginIOError(String::from(format!("Error loading plugin: {}", error)))
	}
}

impl From<string::FromUtf8Error> for CuckooMinerError {
	fn from(error: string::FromUtf8Error) -> Self {
		CuckooMinerError::PluginIOError(String::from(format!(
			"Error loading plugin description: {}",
			error
		)))
	}
}
