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

//! Logging configuration types

/// Log level types, as slog's don't implement serialize
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogLevel {
	/// Critical
	Critical,
	/// Error
	Error,
	/// Warning
	Warning,
	/// Info
	Info,
	/// Debug
	Debug,
	/// Trace
	Trace,
}

/// Logging config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
	/// whether to log to stdout
	pub log_to_stdout: bool,
	/// logging level for stdout
	pub stdout_log_level: LogLevel,
	/// whether to log to file
	pub log_to_file: bool,
	/// log file level
	pub file_log_level: LogLevel,
	/// Log file path
	pub log_file_path: String,
	/// Whether to append to log or replace
	pub log_file_append: bool,
	/// Whether the tui is running (optional)
	pub tui_running: Option<bool>,
}

impl Default for LoggingConfig {
	fn default() -> LoggingConfig {
		LoggingConfig {
			log_to_stdout: true,
			stdout_log_level: LogLevel::Debug,
			log_to_file: false,
			file_log_level: LogLevel::Trace,
			log_file_path: String::from("grin.log"),
			log_file_append: false,
			tui_running: None,
		}
	}
}
