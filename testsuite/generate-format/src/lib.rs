// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! How and where to record the Serde format of interesting Libra types.
//! See API documentation with `cargo doc -p serde-reflection --open`

use serde_reflection::Registry;
use std::str::FromStr;
use structopt::{clap::arg_enum, StructOpt};

/// consensus types.
mod consensus;
/// Libra transactions.
mod libra;

arg_enum! {
#[derive(Debug, StructOpt, Clone, Copy)]
/// A corpus of Rust types to trace, and optionally record on disk.
pub enum Corpus {
    Libra,
    Consensus,
}
}

impl Corpus {
    /// All corpuses.
    pub fn values() -> Vec<Corpus> {
        Corpus::variants()
            .iter()
            .filter_map(|s| Corpus::from_str(s).ok())
            .collect()
    }

    /// Compute the registry of formats.
    pub fn get_registry(self) -> Registry {
        match self {
            Corpus::Libra => libra::get_registry().unwrap(),
            Corpus::Consensus => consensus::get_registry().unwrap(),
        }
    }

    /// Where to record this corpus on disk.
    pub fn output_file(self) -> Option<&'static str> {
        match self {
            Corpus::Libra => libra::output_file(),
            Corpus::Consensus => consensus::output_file(),
        }
    }
}
