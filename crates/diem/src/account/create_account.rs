// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0
use crate::common::types::{CliError, Command};

use async_trait::async_trait;
use clap::Parser;

use crate::common::utils::{generate_key_pair, save_keypair};

/// Create a new local account
///
/// This command generates local account and returns a public/private keypair.
/// The account can be created on chain by transferring coins to the created account.
#[derive(Debug, Parser)]
pub struct CreateAccount {}

#[async_trait]
impl Command<String> for CreateAccount {
    fn command_name(&self) -> &'static str {
        "CreateAccount"
    }

    async fn execute(self) -> Result<String, CliError> {
        let keypair = generate_key_pair(None);

        match save_keypair(keypair) {
            Ok(res) => return Ok(res),
            Err(err) => panic!("Error saving keypair {}", err),
        };
    }
}
