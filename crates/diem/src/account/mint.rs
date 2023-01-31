// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0
use crate::common::{
    types::{CliError, Command},
    utils::mint_new_account,
};

use async_trait::async_trait;
use clap::Parser;

/// Mint coins to an account
///
/// This mints coins to an account.
/// If the account does not exist on chain it will be created.
#[derive(Debug, Parser)]
pub struct MintAccount {
    ///Account to receive coins, authentication key is required to mint from faucet
    account: String,

    ///Amount of coins to mint
    amount: u64,

    ///Currency of coins to mint
    currency: String,
}

#[async_trait]
impl Command<String> for MintAccount {
    fn command_name(&self) -> &'static str {
        "MintAccount"
    }

    async fn execute(self) -> Result<String, CliError> {
        mint_new_account(self.account, self.amount, self.currency).await
    }
}
