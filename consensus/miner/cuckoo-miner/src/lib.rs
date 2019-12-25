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

//! # Overview
//!
//! <b>cuckoo-miner</b> is a Rust wrapper around John Tromp's Cuckoo Miner
//! C implementations, intended primarily for use in the Grin MimbleWimble
//! blockhain development project. However, it is also suitable for use as
//! a standalone miner or by any other project needing to use the
//! cuckoo cycle proof of work. cuckoo-miner is plugin based, and provides
//! a high level interface to load and work with C mining implementations.

#![deny(non_upper_case_globals)]
#![deny(non_camel_case_types)]
#![deny(non_snake_case)]
#![deny(unused_mut)]
#![warn(missing_docs)]

extern crate grin_miner_plugin as plugin;
extern crate grin_miner_util as util;

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

extern crate blake2_rfc as blake2;
extern crate byteorder;
extern crate crypto;
extern crate rand;
extern crate regex;

extern crate libc;
extern crate libloading as libloading;

#[macro_use]
extern crate slog;

extern crate glob;

mod config;
mod cuckoo_sys;
mod error;
mod miner;

pub use config::types::PluginConfig;
pub use cuckoo_sys::ffi::PluginLibrary;
pub use error::error::CuckooMinerError;
pub use miner::miner::CuckooMiner;
pub use plugin::*;
