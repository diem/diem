// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! How and where to record the Serde format of interesting Libra types.
//! See API documentation with `cargo doc -p serde-reflection --open`

use serde_reflection::Registry;
use std::str::FromStr;
use structopt::{clap::arg_enum, StructOpt};

/// Libra transactions.
mod libra;

/// Move file format.
mod moveff;

arg_enum! {
#[derive(Debug, StructOpt, Clone, Copy)]
/// A corpus of Rust types to trace, and optionally record on disk.
pub enum Corpus {
    Libra,
    MoveFileFormat,
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
    pub fn get_registry(self, skip_deserialize: bool) -> Registry {
        match self {
            Corpus::Libra => libra::get_registry(self.to_string(), skip_deserialize),
            Corpus::MoveFileFormat => moveff::get_registry(self.to_string(), skip_deserialize),
        }
    }

    /// Where to record this corpus on disk.
    pub fn output_file(self) -> Option<&'static str> {
        match self {
            Corpus::Libra => libra::output_file(),
            Corpus::MoveFileFormat => moveff::output_file(),
        }
    }
}
