// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_management::{config::ConfigPath, error::Error, secure_backend::ValidatorBackend};
use libra_types::account_address::AccountAddress;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct PrintAccount {
    #[structopt(flatten)]
    config: ConfigPath,
    /// The account name in storage
    #[structopt(long)]
    account_name: String,
    #[structopt(flatten)]
    validator_backend: ValidatorBackend,
}

impl PrintAccount {
    pub fn execute(self) -> Result<AccountAddress, Error> {
        let config = self
            .config
            .load()?
            .override_validator_backend(&self.validator_backend.validator_backend)?;

        let storage = config.validator_backend();
        let account_name = Box::leak(self.account_name.into_boxed_str());
        storage.account_address(account_name)
    }
}
