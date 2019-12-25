// Copyright 2018 The Grin Developers
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

//! Logging wrapper to be used throughout all crates in the workspace
use slog::{Discard, Drain, Duplicate, Level, LevelFilter, Logger};
use slog_async;
use slog_term;
use std::fs::OpenOptions;
use std::ops::Deref;
use std::sync::Mutex;

use backtrace::Backtrace;
use std::{panic, thread};

use types::{LogLevel, LoggingConfig};

fn convert_log_level(in_level: &LogLevel) -> Level {
	match *in_level {
		LogLevel::Info => Level::Info,
		LogLevel::Critical => Level::Critical,
		LogLevel::Warning => Level::Warning,
		LogLevel::Debug => Level::Debug,
		LogLevel::Trace => Level::Trace,
		LogLevel::Error => Level::Error,
	}
}

lazy_static! {
	/// Flag to observe whether logging was explicitly initialised (don't output otherwise)
	static ref WAS_INIT: Mutex<bool> = Mutex::new(false);
	/// Flag to observe whether tui is running, and we therefore don't want to attempt to write
	/// panics to stdout
	static ref TUI_RUNNING: Mutex<bool> = Mutex::new(false);
	/// Static Logging configuration, should only be set once, before first logging call
	static ref LOGGING_CONFIG: Mutex<LoggingConfig> = Mutex::new(LoggingConfig::default());

	/// And a static reference to the logger itself, accessible from all crates
	pub static ref LOGGER: Logger = {
		let was_init = WAS_INIT.lock().unwrap().clone();
		let config = LOGGING_CONFIG.lock().unwrap();
		let slog_level_stdout = convert_log_level(&config.stdout_log_level);
		let slog_level_file = convert_log_level(&config.file_log_level);
		if config.tui_running.is_some() && config.tui_running.unwrap() {
			let mut tui_running_ref = TUI_RUNNING.lock().unwrap();
			*tui_running_ref = true;
		}

		//Terminal output drain
		let terminal_decorator = slog_term::TermDecorator::new().build();
		let terminal_drain = slog_term::FullFormat::new(terminal_decorator).build().fuse();
		let terminal_drain = LevelFilter::new(terminal_drain, slog_level_stdout).fuse();
		let mut terminal_drain = slog_async::Async::new(terminal_drain).build().fuse();
		if !config.log_to_stdout || !was_init {
			terminal_drain = slog_async::Async::new(Discard{}).build().fuse();
		}

		//File drain
		let mut file_drain_final = slog_async::Async::new(Discard{}).build().fuse();
		if config.log_to_file && was_init {
			let file = OpenOptions::new()
				.create(true)
				.write(true)
				.append(config.log_file_append)
				.truncate(!config.log_file_append)
				.open(&config.log_file_path)
				.unwrap();

			let file_decorator = slog_term::PlainDecorator::new(file);
			let file_drain = slog_term::FullFormat::new(file_decorator).build().fuse();
			let file_drain = LevelFilter::new(file_drain, slog_level_file).fuse();
			file_drain_final = slog_async::Async::new(file_drain).build().fuse();
		}

		//Compose file and terminal drains
		let composite_drain = Duplicate::new(terminal_drain, file_drain_final).fuse();

		let log = Logger::root(composite_drain, o!());
		log
	};
}

/// Initialises the logger with the given configuration
pub fn init_logger(config: Option<LoggingConfig>) {
	if let Some(c) = config {
		let mut config_ref = LOGGING_CONFIG.lock().unwrap();
		*config_ref = c.clone();
		// Logger configuration successfully injected into LOGGING_CONFIG...
		let mut was_init_ref = WAS_INIT.lock().unwrap();
		*was_init_ref = true;
		// .. allow logging, having ensured that paths etc are immutable
	}
	send_panic_to_log();
}

/// Initializes the logger for unit and integration tests
pub fn init_test_logger() {
	let mut was_init_ref = WAS_INIT.lock().unwrap();
	if *was_init_ref.deref() {
		return;
	}
	let mut config_ref = LOGGING_CONFIG.lock().unwrap();
	*config_ref = LoggingConfig::default();
	*was_init_ref = true;
	send_panic_to_log();
}

/// hook to send panics to logs as well as stderr
fn send_panic_to_log() {
	panic::set_hook(Box::new(|info| {
		let backtrace = Backtrace::new();

		let thread = thread::current();
		let thread = thread.name().unwrap_or("unnamed");

		let msg = match info.payload().downcast_ref::<&'static str>() {
			Some(s) => *s,
			None => match info.payload().downcast_ref::<String>() {
				Some(s) => &**s,
				None => "Box<Any>",
			},
		};

		match info.location() {
			Some(location) => {
				error!(
					LOGGER,
					"\nthread '{}' panicked at '{}': {}:{}{:?}\n\n",
					thread,
					msg,
					location.file(),
					location.line(),
					backtrace
				);
			}
			None => error!(
				LOGGER,
				"thread '{}' panicked at '{}'{:?}", thread, msg, backtrace
			),
		}
		//also print to stderr
		let tui_running = TUI_RUNNING.lock().unwrap().clone();
		if !tui_running {
			eprintln!(
				"Thread '{}' panicked with message:\n\"{}\"\nSee grin.log for further details.",
				thread, msg
			);
		}
	}));
}
