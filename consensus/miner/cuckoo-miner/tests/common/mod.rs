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
//

//! Common values and functions that can be used in all mining tests

extern crate cuckoo_miner as cuckoo;
extern crate time;

use self::cuckoo::{CuckooMiner, PluginConfig};
use std;
use std::env;
use std::path::PathBuf;

/// Values from T4 genesis that should be validated
pub const T4_GENESIS_PREPOW: &str =
    "00010000000000000000000000005bc794c0fffffffff\
     fffffffffffffffffffffffffffffffffffffffffffffffffffffff0000000000000000000000\
     00000000000000000000000000000000000000000000000000000000000000000000000000000\
     00000000000000000000000000000000000000000000000000000000000000000000000000000\
     00000000000000000000000000000000000000000000000000000000000000000000000000000\
     00000000000000000000000000000000000000000000000000000000000000000000000000000\
     000000000000000000000000000000001c5200000007407784d410a210b1ba";

pub const _T4_GENESIS_NONCE: u64 = 8612241555342799290;
pub const T4_GENESIS_PROOF: [u64; 42] = [
    0x46f3b4, 0x1135f8c, 0x1a1596f, 0x1e10f71, 0x41c03ea, 0x63fe8e7, 0x65af34f, 0x73c16d3,
    0x8216dc3, 0x9bc75d0, 0xae7d9ad, 0xc1cb12b, 0xc65e957, 0xf67a152, 0xfac6559, 0x100c3d71,
    0x11eea08b, 0x1225dfbb, 0x124d61a1, 0x132a14b4, 0x13f4ec38, 0x1542d236, 0x155f2df0, 0x1577394e,
    0x163c3513, 0x19349845, 0x19d46953, 0x19f65ed4, 0x1a0411b9, 0x1a2fa039, 0x1a72a06c, 0x1b02ddd2,
    0x1b594d59, 0x1b7bffd3, 0x1befe12e, 0x1c82e4cd, 0x1d492478, 0x1de132a5, 0x1e578b3c, 0x1ed96855,
    0x1f222896, 0x1fea0da6,
];

// Helper function, derives the plugin directory for mining tests
pub fn mining_plugin_dir_for_tests() -> PathBuf {
    env::current_exe()
        .map(|mut env_path| {
            env_path.pop();
            // cargo test exes are a directory further down
            if env_path.ends_with("deps") {
                env_path.pop();
            }
            env_path.push("plugins");
            env_path
        })
        .unwrap()
}

// Helper function, tests a particular miner implementation against a known set
pub fn mine_async_for_duration(configs: &Vec<PluginConfig>, duration_in_seconds: i64) {
    let stat_check_interval = 3;
    let mut deadline = time::get_time().sec + duration_in_seconds;
    let mut next_stat_check = time::get_time().sec + stat_check_interval;
    let mut stats_updated = false;

    //for CI testing on slower servers
    //if we're trying to quit and there are no stats yet, keep going for a bit
    let mut extra_time = false;
    let extra_time_value = 600;

    // these always get consumed after a notify
    let mut miner = CuckooMiner::new(configs.clone());
    let _ = miner.start_solvers();

    while time::get_time().sec < deadline {
        println!(
            "Test mining for {} seconds, looking for difficulty >= 0",
            duration_in_seconds
        );
        let mut i = 0;
        for c in configs.clone().into_iter() {
            println!("Plugin {}: {}", i, c.name);
            i += 1;
        }

        miner.notify(1, 1, T4_GENESIS_PREPOW, "", 0).unwrap();

        loop {
            if let Some(solutions) = miner.get_solutions() {
                for i in 0..solutions.num_sols {
                    println!("Sol found: {}", solutions.sols[i as usize]);
                    continue;
                }
            }
            if time::get_time().sec >= next_stat_check {
                let mut sps_total = 0.0;
                let stats_vec = miner.get_stats();
                for s in stats_vec.unwrap().into_iter() {
                    let last_solution_time_secs = s.last_solution_time as f64 / 1000000000.0;
                    let last_hashes_per_sec = 1.0 / last_solution_time_secs;
                    let status = match s.has_errored {
                        false => "OK",
                        _ => "ERRORED",
                    };
                    println!("Plugin 0 - Device {} ({}) (Cuck(at)oo{}) - Status: {} - Last Graph time: {}; Graphs per second: {:.*} \
					- Total Attempts: {}",
					s.device_id, s.get_device_name(), s.edge_bits, status, last_solution_time_secs, 3, last_hashes_per_sec, s.iterations);
                    if last_hashes_per_sec.is_finite() {
                        sps_total += last_hashes_per_sec;
                    }
                    if last_solution_time_secs > 0.0 {
                        stats_updated = true;
                    }
                    i += 1;
                }
                println!("Total solutions per second: {}", sps_total);
                next_stat_check = time::get_time().sec + stat_check_interval;
            }
            if time::get_time().sec > deadline {
                if !stats_updated && !extra_time {
                    extra_time = true;
                    deadline += extra_time_value;
                    println!("More time needed");
                } else {
                    println!("Stopping jobs and waiting for cleanup");
                    miner.stop_solvers();
                    break;
                }
            }
            if stats_updated && extra_time {
                miner.stop_solvers();
                return;
            }
            //avoid busy wait
            let sleep_dur = std::time::Duration::from_millis(100);
            std::thread::sleep(sleep_dur);
            if stats_updated && extra_time {
                break;
            }
        }
    }
}
