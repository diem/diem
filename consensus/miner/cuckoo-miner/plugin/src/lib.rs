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

//! Crate wrapping up the Grin miner plugins

#![deny(non_upper_case_globals)]
#![deny(non_camel_case_types)]
#![deny(non_snake_case)]
#![deny(unused_mut)]
#![warn(missing_docs)]

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate blake2_rfc as blake2;
extern crate byteorder;
extern crate libc;
extern crate serde_json;

use libc::*;
use std::ffi::CString;
use std::ptr::NonNull;
use std::{cmp, fmt, marker};

use blake2::blake2b::Blake2b;
use byteorder::{BigEndian, ByteOrder};

/// Size of proof
pub const PROOFSIZE: usize = 42;

/// Maximin length of plugin name w
pub const MAX_NAME_LEN: usize = 256;
/// Maximum number of solutions
pub const MAX_SOLS: usize = 4;

// Type definitions corresponding to each function that the plugin/solver implements
/// Create solver function
pub type CuckooCreateSolverCtx = unsafe extern "C" fn(*mut SolverParams) -> *mut SolverCtx;
/// Destroy solver function
pub type CuckooDestroySolverCtx = unsafe extern "C" fn(*mut SolverCtx);
/// Run solver function
pub type CuckooRunSolver = unsafe extern "C" fn(
    *mut SolverCtx,       // Solver context
    *const c_uchar,       // header
    uint32_t,             // header length
    uint64_t,             // nonce
    uint32_t,             // range
    *mut SolverSolutions, // reference to any found solutions
    *mut SolverStats,     // solver stats
) -> uint32_t;
/// Stop solver function
pub type CuckooStopSolver = unsafe extern "C" fn(*mut SolverCtx);
/// Fill default params of solver
pub type CuckooFillDefaultParams = unsafe extern "C" fn(*mut SolverParams);

/// A solver context, opaque reference to C++ type underneath
#[derive(Copy, Clone, Debug)]
pub enum SolverCtx {}
/// wrap ctx to send across threads
pub struct SolverCtxWrapper(pub NonNull<SolverCtx>);
unsafe impl marker::Send for SolverCtxWrapper {}

/// Common parameters for a solver
#[derive(Clone, Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct SolverParams {
    /// threads
    pub nthreads: uint32_t,
    /// trims
    pub ntrims: uint32_t,
    /// Whether to show cycle (should be true to get solutions)
    pub showcycle: bool,
    /// allrounds
    pub allrounds: bool,
    /// whether to apply the nonce to the header, or leave as is,
    /// letting caller mutate nonce
    pub mutate_nonce: bool,
    /// reduce cpuload
    pub cpuload: bool,

    /// Common Cuda params
    pub device: u32,

    /// Lean cuda params
    pub blocks: u32,
    ///
    pub tpb: u32,

    /// Mean cuda params
    pub expand: u32,
    ///
    pub genablocks: u32,
    ///
    pub genatpb: u32,
    ///
    pub genbtpb: u32,
    ///
    pub trimtpb: u32,
    ///
    pub tailtpb: u32,
    ///
    pub recoverblocks: u32,
    ///
    pub recovertpb: u32,
    /// OCL platform ID, 0 - default, 1 - AMD, 2 - NVIDIA
    pub platform: u32,
    /// edge bits for OCL plugins
    pub edge_bits: u32,
}

impl Default for SolverParams {
    fn default() -> SolverParams {
        SolverParams {
            nthreads: 0,
            ntrims: 0,
            showcycle: true,
            allrounds: false,
            mutate_nonce: false,
            cpuload: true,
            device: 0,
            blocks: 0,
            tpb: 0,
            expand: 0,
            genablocks: 0,
            genatpb: 0,
            genbtpb: 0,
            trimtpb: 0,
            tailtpb: 0,
            recoverblocks: 0,
            recovertpb: 0,
            platform: 0,
            edge_bits: 31,
        }
    }
}

/// Common stats collected by solvers
#[derive(Clone)]
#[repr(C)]
pub struct SolverStats {
    /// device Id
    pub device_id: uint32_t,
    /// graph size
    pub edge_bits: uint32_t,
    /// plugin name
    pub plugin_name: [c_uchar; MAX_NAME_LEN],
    /// device name
    pub device_name: [c_uchar; MAX_NAME_LEN],
    /// whether device has reported an error
    pub has_errored: bool,
    /// reason for error
    pub error_reason: [c_uchar; MAX_NAME_LEN],
    /// number of searched completed by device
    pub iterations: uint32_t,
    /// last solution start time
    pub last_start_time: uint64_t,
    /// last solution end time
    pub last_end_time: uint64_t,
    /// last solution elapsed time
    pub last_solution_time: uint64_t,
}

impl Default for SolverStats {
    fn default() -> SolverStats {
        SolverStats {
            device_id: 0,
            edge_bits: 0,
            plugin_name: [0; MAX_NAME_LEN],
            device_name: [0; MAX_NAME_LEN],
            has_errored: false,
            error_reason: [0; MAX_NAME_LEN],
            iterations: 0,
            last_start_time: 0,
            last_end_time: 0,
            last_solution_time: 0,
        }
    }
}

impl SolverStats {
    fn get_name(&self, c_str: &[u8; MAX_NAME_LEN]) -> String {
        // remove all null zeroes
        let v = c_str.clone().to_vec();
        let mut i = 0;
        for j in 0..v.len() {
            if v.get(j) == Some(&0) {
                i = j;
                break;
            }
        }
        let v = v.split_at(i).0;
        match CString::new(v) {
            Ok(s) => s.to_str().unwrap().to_owned(),
            Err(_) => String::from("Unknown Device Name"),
        }
    }
    /// return device name as rust string
    pub fn get_device_name(&self) -> String {
        self.get_name(&self.device_name)
    }
    /// return plugin name as rust string
    pub fn get_plugin_name(&self) -> String {
        self.get_name(&self.plugin_name)
    }
    /// return plugin name as rust string
    pub fn get_error_reason(&self) -> String {
        self.get_name(&self.error_reason)
    }
    /// set plugin name
    pub fn set_plugin_name(&mut self, name: &str) {
        let c_vec = CString::new(name).unwrap().into_bytes();
        for i in 0..c_vec.len() {
            self.plugin_name[i] = c_vec[i];
        }
    }
}

/// A single solution
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Solution {
    /// Optional ID
    pub id: uint64_t,
    /// Nonce
    pub nonce: uint64_t,
    /// Proof
    pub proof: [uint64_t; PROOFSIZE],
}

impl Default for Solution {
    fn default() -> Solution {
        Solution {
            id: 0,
            nonce: 0,
            proof: [0u64; PROOFSIZE],
        }
    }
}

impl fmt::Display for Solution {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut comma_separated = String::new();

        for num in &self.proof[0..self.proof.len()] {
            comma_separated.push_str(&format!("0x{:X}", &num));
            comma_separated.push_str(", ");
        }
        comma_separated.pop();
        comma_separated.pop();

        write!(f, "Nonce:{} [{}]", self.nonce, comma_separated)
    }
}

impl fmt::Debug for Solution {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", &self.proof[..])
    }
}

impl cmp::PartialEq for Solution {
    fn eq(&self, other: &Solution) -> bool {
        for i in 0..PROOFSIZE {
            if self.proof[i] != other.proof[i] {
                return false;
            }
        }
        return true;
    }
}

impl Solution {
    /// Converts the proof to a vector of u64s
    pub fn to_u64s(&self) -> Vec<u64> {
        let mut nonces = Vec::with_capacity(PROOFSIZE);
        for n in self.proof.iter() {
            nonces.push(*n as u64);
        }
        nonces
    }

    /// Returns the hash of the solution
    pub fn hash(&self) -> [u8; 32] {
        // Hash
        let mut blake2b = Blake2b::new(32);
        for n in 0..self.proof.len() {
            let mut bytes = [0; 4];
            BigEndian::write_u32(&mut bytes, self.proof[n] as u32);
            blake2b.update(&bytes);
        }
        let mut ret = [0; 32];
        ret.copy_from_slice(blake2b.finalize().as_bytes());
        ret
    }
}

/// All solutions returned
#[derive(Clone, Copy)]
#[repr(C)]
pub struct SolverSolutions {
    /// graph size
    pub edge_bits: u32,
    /// number of solutions
    pub num_sols: u32,
    /// solutions themselves
    pub sols: [Solution; MAX_SOLS],
}

impl Default for SolverSolutions {
    fn default() -> SolverSolutions {
        SolverSolutions {
            edge_bits: 0,
            num_sols: 0,
            sols: [Solution::default(); MAX_SOLS],
        }
    }
}
