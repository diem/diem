// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::common::types::Command;
use clap::Subcommand;

pub mod create_account;

/// Tool to interact with account data
///
/// This is used to create, request information and facilitate transfers between accounts.
#[derive(Debug, Subcommand)]
pub enum AccountSubcommand {
    Create(create_account::CreateAccount),
}

impl AccountSubcommand {
    pub async fn execute(self) -> Result<String, String> {
        match self {
            AccountSubcommand::Create(tool) => tool.execute_serialized().await,
        }
    }
}
