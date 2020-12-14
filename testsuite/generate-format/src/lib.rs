// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! How and where to record the Serde format of interesting Diem types.
//! See API documentation with `cargo doc -p serde-reflection --open`

use serde_reflection::Registry;
use std::str::FromStr;
use structopt::{clap::arg_enum, StructOpt};

/// Consensus messages.
mod consensus;
/// Diem transactions.
mod diem;
/// Analyze Serde formats to detect certain patterns.
mod linter;
/// Move ABI.
mod move_abi;
/// Network messages.
mod network;

pub use linter::lint_bcs_format;

arg_enum! {
#[derive(Debug, StructOpt, Clone, Copy)]
/// A corpus of Rust types to trace, and optionally record on disk.
pub enum Corpus {
    Diem,
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
            Corpus::Diem => diem::get_registry(),
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
            Corpus::Diem => diem::output_file(),
            Corpus::Consensus => consensus::output_file(),
            Corpus::Network => network::output_file(),
            Corpus::MoveABI => move_abi::output_file(),
        }
    }
}
