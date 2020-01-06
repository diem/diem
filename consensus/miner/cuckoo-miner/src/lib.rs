// Copyright 2018 The Grin Developers

//! cuckoo-miner is a Rust wrapper around John Tromp's Cuckoo Miner
//! C implementations, intended primarily for use in the Grin MimbleWimble
//! blockhain development project. However, it is also suitable for use as
//! a standalone miner or by any other project needing to use the
//! cuckoo cycle proof of work. cuckoo-miner is plugin based, and provides
//! a high level interface to load and work with C mining implementations.

extern crate cuckoo_plugin as plugin;

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

extern crate blake2_rfc as blake2;
extern crate byteorder;
extern crate crypto;
extern crate glob;
extern crate libc;
extern crate libloading;
extern crate rand;
extern crate regex;
extern crate slog;

mod config;
mod cuckoo_sys;
mod error;

pub use config::types::PluginConfig;
pub use cuckoo_sys::ffi::PluginLibrary;
pub use error::error::CuckooMinerError;
pub use plugin::*;
