// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use diem_global_constants::OWNER_ACCOUNT;
use diem_management::{constants, error::Error, secure_backend::SharedBackend};
use diem_types::{network_address::NetworkAddress, transaction::Transaction};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct ValidatorConfig {
    #[structopt(long)]
    owner_name: String,
    #[structopt(flatten)]
    validator_config: diem_management::validator_config::ValidatorConfig,
    #[structopt(long)]
    validator_address: NetworkAddress,
    #[structopt(long)]
    fullnode_address: NetworkAddress,
    #[structopt(flatten)]
    shared_backend: SharedBackend,
    #[structopt(long, help = "Disables network address validation")]
    disable_address_validation: bool,
}

impl ValidatorConfig {
    pub fn execute(self) -> Result<Transaction, Error> {
        let config = self
            .validator_config
            .config()?
            .override_shared_backend(&self.shared_backend.shared_backend)?;

        // Retrieve and set owner account
        let owner_account =
            diem_config::utils::validator_owner_account_from_name(self.owner_name.as_bytes());
        let mut validator_storage = config.validator_backend();
        validator_storage.set(OWNER_ACCOUNT, owner_account)?;

        let txn = self.validator_config.build_transaction(
            0,
            self.fullnode_address,
            self.validator_address,
            false,
            self.disable_address_validation,
        )?;

        // Upload the validator config to shared storage
        let mut shared_storage = config.shared_backend();
        shared_storage.set(constants::VALIDATOR_CONFIG, txn.clone())?;

        Ok(txn)
    }
}
