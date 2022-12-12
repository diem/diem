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

// Command Line Interface (CLI) for interacting with the Diem blockchain
#[derive(Parser)]
#[clap(name = "diem", author, version, propagate_version = true)]
pub enum Tool {
    #[clap(subcommand)]
    Account(account::AccountSubcommand),
    #[clap(subcommand)]
    Config(config::ConfigTool),
}

impl Tool {
    pub async fn execute(self) -> Result<String, String> {
        use Tool::*;
        match self {
            Account(tool) => tool.execute().await,
            Config(tool) => tool.execute().await,
        }
    }
}
