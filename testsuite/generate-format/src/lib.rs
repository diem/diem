// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! How and where to record the Serde format of interesting Libra types.
//! See API documentation with `cargo doc -p serde-reflection --open`

use serde_reflection::Registry;
use std::str::FromStr;
use structopt::{clap::arg_enum, StructOpt};

/// Consensus messages.
mod consensus;
/// Libra transactions.
mod libra;
/// Analyze Serde formats to detect certain patterns.
mod linter;
/// Move ABI.
mod move_abi;
/// Network messages.
mod network;

pub use linter::lint_lcs_format;

arg_enum! {
#[derive(Debug, StructOpt, Clone, Copy)]
/// A corpus of Rust types to trace, and optionally record on disk.
pub enum Corpus {
    Libra,
    Consensus,
    Network,
    MoveABI,
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
        let result = match self {
            Corpus::Libra => libra::get_registry(),
            Corpus::Consensus => consensus::get_registry(),
            Corpus::Network => network::get_registry(),
            Corpus::MoveABI => move_abi::get_registry(),
        };
        match result {
            Ok(registry) => registry,
            Err(error) => {
                panic!("{}:{}", error, error.explanation());
            }
        }
    }

    /// Where to record this corpus on disk.
    pub fn output_file(self) -> Option<&'static str> {
        match self {
            Corpus::Libra => libra::output_file(),
            Corpus::Consensus => consensus::output_file(),
            Corpus::Network => network::output_file(),
            Corpus::MoveABI => move_abi::output_file(),
        }
    }
}
