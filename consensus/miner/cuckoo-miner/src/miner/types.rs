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

//! Miner types
use std::sync::{Arc, RwLock};

use plugin::{SolverSolutions, SolverStats};
use CuckooMinerError;
use {PluginConfig, PluginLibrary};

pub type JobSharedDataType = Arc<RwLock<JobSharedData>>;

/// Holds a loaded lib + config + stats
/// 1 instance = 1 device on 1 controlling thread
pub struct SolverInstance {
	/// The loaded plugin
	pub lib: PluginLibrary,
	/// Associated config
	pub config: PluginConfig,
	/// Last stats output
	pub stats: SolverStats,
	/// Last solution output
	pub solutions: SolverSolutions,
}

impl SolverInstance {
	/// Create a new solver instance with the given config
	pub fn new(config: PluginConfig) -> Result<SolverInstance, CuckooMinerError> {
		let l = PluginLibrary::new(&config.file)?;
		Ok(SolverInstance {
			lib: l,
			config: config,
			stats: SolverStats::default(),
			solutions: SolverSolutions::default(),
		})
	}

	/// Release the lib
	pub fn unload(&mut self) {
		self.lib.unload();
	}
}

/// Data intended to be shared across threads
pub struct JobSharedData {
	/// ID of the current running job (not currently used)
	pub job_id: u32,

	/// block height of current running job
	pub height: u64,

	/// The part of the header before the nonce, which this
	/// module will mutate in search of a solution
	pub pre_nonce: String,

	/// The part of the header after the nonce
	pub post_nonce: String,

	/// The target difficulty. Only solutions >= this
	/// target will be put into the output queue
	pub difficulty: u64,

	/// Output solutions
	pub solutions: Vec<SolverSolutions>,

	/// Current stats
	pub stats: Vec<SolverStats>,
}

impl Default for JobSharedData {
	fn default() -> JobSharedData {
		JobSharedData {
			job_id: 0,
			height: 0,
			pre_nonce: String::from(""),
			post_nonce: String::from(""),
			difficulty: 0,
			solutions: Vec::new(),
			stats: vec![],
		}
	}
}

impl JobSharedData {
	pub fn new(num_solvers: usize) -> JobSharedData {
		JobSharedData {
			job_id: 0,
			height: 0,
			pre_nonce: String::from(""),
			post_nonce: String::from(""),
			difficulty: 1,
			solutions: Vec::new(),
			stats: vec![SolverStats::default(); num_solvers],
		}
	}
}
