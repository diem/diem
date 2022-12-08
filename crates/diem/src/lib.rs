// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

pub mod account;
pub mod common;
pub mod config;
pub mod genesis;
pub mod key;
pub mod move_tool;
pub mod node;

use clap::Parser;
// use std::collections::BTreeMap;

// Command Line Interface (CLI) for developing and interacting with the Diem blockchain
#[derive(Parser)]
#[clap(name = "diem", author, version, propagate_version = true)]
pub enum Tool {
    #[clap(subcommand)]
    Config(config::ConfigTool),
}

impl Tool {
    pub async fn execute(self) -> Result<String, String> {
        use Tool::*;
        match self {
            Config(tool) => tool.execute().await,
        }
    }
}
