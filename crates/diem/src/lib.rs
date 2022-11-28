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
    // #[clap(subcommand)]
    // Account(account::AccountTool),
    // #[clap(subcommand)]
    // Config(config::ConfigTool),
    // #[clap(subcommand)]
    // Genesis(genesis::GenesisTool),
    // Info(InfoTool),
    // Init(common::init::InitTool),
    // #[clap(subcommand)]
    // Key(key::KeyTool),
    // #[clap(subcommand)]
    // Move(move_tool::MoveTool),
    // #[clap(subcommand)]
    // Node(node::NodeTool),
}

impl Tool {
    pub async fn execute(self) -> Result<String, String> {
        // use Tool::*;
        match self {
            // Account(tool) => tool.execute().await,
            // Config(tool) => tool.execute().await,
            // Genesis(tool) => tool.execute().await,
            // Info(tool) => tool.execute_serialized().await,
            // Init(tool) => tool.execute_serialized_success().await,
            // Key(tool) => tool.execute().await,
            // Move(tool) => tool.execute().await,
            // Node(tool) => tool.execute().await,
        }
    }
}