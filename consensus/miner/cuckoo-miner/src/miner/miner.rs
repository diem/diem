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

//! Main interface for callers into cuckoo-miner. Provides functionality
//! to load a mining plugin, send it a Cuckoo Cycle POW problem, and
//! return any resulting solutions.

use std::ptr::NonNull;
use std::sync::{mpsc, Arc, RwLock};
use std::{thread, time};
use util::LOGGER;

use config::types::PluginConfig;
use miner::types::{JobSharedData, JobSharedDataType, SolverInstance};

use miner::consensus::Proof;
use miner::util;
use plugin::{Solution, SolverCtxWrapper, SolverSolutions, SolverStats};
use {CuckooMinerError, PluginLibrary};

/// Miner control Messages
#[derive(Debug)]
enum ControlMessage {
	/// Stop everything, pull down, exis
	Stop,
	/// Stop current mining iteration, set solver threads to paused
	Pause,
	/// Resume
	Resume,
	/// Solver reporting stopped
	SolverStopped(usize),
}

/// An instance of a miner, which loads a cuckoo-miner plugin
/// and calls its mine function according to the provided configuration

pub struct CuckooMiner {
	/// Configurations
	configs: Vec<PluginConfig>,

	/// Data shared across threads
	pub shared_data: Arc<RwLock<JobSharedData>>,

	/// Job control tx
	control_txs: Vec<mpsc::Sender<ControlMessage>>,

	/// solver loop tx
	solver_loop_txs: Vec<mpsc::Sender<ControlMessage>>,

	/// Solver has stopped and cleanly shutdown
	solver_stopped_rxs: Vec<mpsc::Receiver<ControlMessage>>,
}

impl CuckooMiner {
	/// Creates a new instance of a CuckooMiner with the given configuration.
	/// One PluginConfig per device

	pub fn new(configs: Vec<PluginConfig>) -> CuckooMiner {
		let len = configs.len();
		CuckooMiner {
			configs: configs,
			shared_data: Arc::new(RwLock::new(JobSharedData::new(len))),
			control_txs: vec![],
			solver_loop_txs: vec![],
			solver_stopped_rxs: vec![],
		}
	}

	/// Solver's instance of a thread
	fn solver_thread(
		mut solver: SolverInstance,
		instance: usize,
		shared_data: JobSharedDataType,
		control_rx: mpsc::Receiver<ControlMessage>,
		solver_loop_rx: mpsc::Receiver<ControlMessage>,
		solver_stopped_tx: mpsc::Sender<ControlMessage>,
	) {
		{
			let mut s = shared_data.write().unwrap();
			s.stats[instance].set_plugin_name(&solver.config.name);
		}
		// "Detach" a stop function from the solver, to let us keep a control thread going
		let ctx = solver.lib.create_solver_ctx(&mut solver.config.params);
		let control_ctx = SolverCtxWrapper(NonNull::new(ctx).unwrap());

		let stop_fn = solver.lib.get_stop_solver_instance();

		// monitor whether to send a stop signal to the solver, which should
		// end the current solve attempt below
		let stop_handle = thread::spawn(move || loop {
			let ctx_ptr = control_ctx.0.as_ptr();
			while let Some(message) = control_rx.iter().next() {
				match message {
					ControlMessage::Stop => {
						PluginLibrary::stop_solver_from_instance(stop_fn.clone(), ctx_ptr);
						return;
					}
					ControlMessage::Pause => {
						PluginLibrary::stop_solver_from_instance(stop_fn.clone(), ctx_ptr);
					}
					_ => {}
				};
			}
		});

		let mut iter_count = 0;
		let mut paused = true;
		loop {
			if let Some(message) = solver_loop_rx.try_iter().next() {
				debug!(
					LOGGER,
					"solver_thread - solver_loop_rx got msg: {:?}", message
				);
				match message {
					ControlMessage::Stop => break,
					ControlMessage::Pause => paused = true,
					ControlMessage::Resume => paused = false,
					_ => {}
				}
			}
			if paused {
				thread::sleep(time::Duration::from_micros(100));
				continue;
			}
			{
				let mut s = shared_data.write().unwrap();
				s.stats[instance].set_plugin_name(&solver.config.name);
			}
			let header_pre = { shared_data.read().unwrap().pre_nonce.clone() };
			let header_post = { shared_data.read().unwrap().post_nonce.clone() };
			let height = { shared_data.read().unwrap().height.clone() };
			let job_id = { shared_data.read().unwrap().job_id.clone() };
			let target_difficulty = { shared_data.read().unwrap().difficulty.clone() };
			let header = util::get_next_header_data(&header_pre, &header_post);
			let nonce = header.0;
			//let sec_scaling = header.2;
			solver.lib.run_solver(
				ctx,
				header.1,
				0,
				1,
				&mut solver.solutions,
				&mut solver.stats,
			);
			iter_count += 1;
			let still_valid = { height == shared_data.read().unwrap().height };
			if still_valid {
				let mut s = shared_data.write().unwrap();
				s.stats[instance] = solver.stats.clone();
				s.stats[instance].iterations = iter_count;
				if solver.solutions.num_sols > 0 {
					// Filter solutions that don't meet difficulty check
					let mut filtered_sols: Vec<Solution> = vec![];
					for i in 0..solver.solutions.num_sols {
						filtered_sols.push(solver.solutions.sols[i as usize]);
					}
					let mut filtered_sols: Vec<Solution> = filtered_sols
						.iter()
						.filter(|s| {
							let proof = Proof {
								edge_bits: solver.solutions.edge_bits as u8,
								nonces: s.proof.to_vec(),
							};
							proof.to_difficulty_unscaled().to_num() >= target_difficulty
						})
						.map(|s| s.clone())
						.collect();
					for mut ss in filtered_sols.iter_mut() {
						ss.nonce = nonce;
						ss.id = job_id as u64;
					}
					solver.solutions.num_sols = filtered_sols.len() as u32;
					for i in 0..solver.solutions.num_sols as usize {
						solver.solutions.sols[i] = filtered_sols[i];
					}
					s.solutions.push(solver.solutions.clone());
				}
				if s.stats[instance].has_errored {
					s.stats[instance].set_plugin_name(&solver.config.name);
					error!(
						LOGGER,
						"Plugin {} has errored, device: {}. Reason: {}",
						s.stats[instance].get_plugin_name(),
						s.stats[instance].get_device_name(),
						s.stats[instance].get_error_reason(),
					);
					break;
				}
			}
			solver.solutions = SolverSolutions::default();
			thread::sleep(time::Duration::from_micros(100));
		}

		let _ = stop_handle.join();
		solver.lib.destroy_solver_ctx(ctx);
		solver.unload();
		let _ = solver_stopped_tx.send(ControlMessage::SolverStopped(instance));
	}

	/// Starts solvers, ready for jobs via job control
	pub fn start_solvers(&mut self) -> Result<(), CuckooMinerError> {
		let mut solvers = Vec::new();
		for c in self.configs.clone() {
			solvers.push(SolverInstance::new(c)?);
		}
		let mut i = 0;
		for s in solvers {
			let sd = self.shared_data.clone();
			let (control_tx, control_rx) = mpsc::channel::<ControlMessage>();
			let (solver_tx, solver_rx) = mpsc::channel::<ControlMessage>();
			let (solver_stopped_tx, solver_stopped_rx) = mpsc::channel::<ControlMessage>();
			self.control_txs.push(control_tx);
			self.solver_loop_txs.push(solver_tx);
			self.solver_stopped_rxs.push(solver_stopped_rx);
			thread::spawn(move || {
				let _ =
					CuckooMiner::solver_thread(s, i, sd, control_rx, solver_rx, solver_stopped_tx);
			});
			i += 1;
		}
		Ok(())
	}

	/// An asynchronous -esque version of the plugin miner, which takes
	/// parts of the header and the target difficulty as input, and begins
	/// asyncronous processing to find a solution. The loaded plugin is
	/// responsible
	/// for how it wishes to manage processing or distribute the load. Once
	/// called
	/// this function will continue to find solutions over the target difficulty
	/// for the given inputs and place them into its output queue until
	/// instructed to stop.

	pub fn notify(
		&mut self,
		job_id: u32,      // Job id
		height: u64,      // Job height
		pre_nonce: &str,  // Pre-nonce portion of header
		post_nonce: &str, // Post-nonce portion of header
		difficulty: u64,  /* The target difficulty, only sols greater than this difficulty will
		                   * be returned. */
	) -> Result<(), CuckooMinerError> {
		let mut sd = self.shared_data.write().unwrap();
		let mut paused = false;
		if height != sd.height {
			// stop/pause any existing jobs if job is for a new
			// height
			self.pause_solvers();
			paused = true;
		}
		sd.job_id = job_id;
		sd.height = height;
		sd.pre_nonce = pre_nonce.to_owned();
		sd.post_nonce = post_nonce.to_owned();
		sd.difficulty = difficulty;
		if paused {
			self.resume_solvers();
		}
		Ok(())
	}

	/// Returns solutions if currently waiting.

	pub fn get_solutions(&self) -> Option<SolverSolutions> {
		// just to prevent endless needless locking of this
		// when using fast test miners, in real cuckoo30 terms
		// this shouldn't be an issue
		// TODO: Make this less blocky
		// let time_pre_lock=Instant::now();
		{
			let mut s = self.shared_data.write().unwrap();
			// let time_elapsed=Instant::now()-time_pre_lock;
			// println!("Get_solution Time spent waiting for lock: {}",
			// time_elapsed.as_secs()*1000 +(time_elapsed.subsec_nanos()/1_000_000)as u64);
			if s.solutions.len() > 0 {
				let sol = s.solutions.pop().unwrap();
				return Some(sol);
			}
		}
		None
	}

	/// get stats for all running solvers
	pub fn get_stats(&self) -> Result<Vec<SolverStats>, CuckooMinerError> {
		let s = self.shared_data.read().unwrap();
		Ok(s.stats.clone())
	}

	/// #Description
	///
	/// Stops the current job, and signals for the loaded plugin to stop
	/// processing and perform any cleanup it needs to do.
	///
	/// #Returns
	///
	/// Nothing

	pub fn stop_solvers(&self) {
		for t in self.control_txs.iter() {
			let _ = t.send(ControlMessage::Stop);
		}
		for t in self.solver_loop_txs.iter() {
			let _ = t.send(ControlMessage::Stop);
		}
		debug!(LOGGER, "Stop message sent");
	}

	/// Tells current solvers to stop and wait
	pub fn pause_solvers(&self) {
		for t in self.control_txs.iter() {
			let _ = t.send(ControlMessage::Pause);
		}
		for t in self.solver_loop_txs.iter() {
			let _ = t.send(ControlMessage::Pause);
		}
		debug!(LOGGER, "Pause message sent");
	}

	/// Tells current solvers to stop and wait
	pub fn resume_solvers(&self) {
		for t in self.control_txs.iter() {
			let _ = t.send(ControlMessage::Resume);
		}
		for t in self.solver_loop_txs.iter() {
			let _ = t.send(ControlMessage::Resume);
		}
		debug!(LOGGER, "Resume message sent");
	}

	/// block until solvers have all exited
	pub fn wait_for_solver_shutdown(&self) {
		for r in self.solver_stopped_rxs.iter() {
			while let Some(message) = r.iter().next() {
				match message {
					ControlMessage::SolverStopped(i) => {
						debug!(LOGGER, "Solver stopped: {}", i);
						break;
					}
					_ => {}
				}
			}
		}
	}
}
