// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_global_constants::{OWNER_ACCOUNT, OWNER_KEY};
use libra_management::{constants, error::Error, secure_backend::SharedBackend};
use libra_network_address::NetworkAddress;
use libra_secure_storage::Value;
use libra_types::{account_address, transaction::Transaction};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct ValidatorConfig {
    #[structopt(long)]
    owner_name: String,
    #[structopt(flatten)]
    validator_config: libra_management::validator_config::ValidatorConfig,
    #[structopt(long)]
    validator_address: NetworkAddress,
    #[structopt(long)]
    fullnode_address: NetworkAddress,
    #[structopt(flatten)]
    shared_backend: SharedBackend,
}

impl ValidatorConfig {
    pub fn execute(self) -> Result<Transaction, Error> {
        let config = self
            .validator_config
            .config
            .load()?
            .override_shared_backend(&self.shared_backend.shared_backend)?
            .override_validator_backend(
                &self.validator_config.validator_backend.validator_backend,
            )?;

        // Retrieve and set owner account
        let owner_shared_storage = config.shared_backend_with_namespace(self.owner_name.clone());
        let owner_key = owner_shared_storage.ed25519_key(OWNER_KEY)?;
        let owner_account = account_address::from_public_key(&owner_key);

        let mut validator_storage = config.validator_backend();
        validator_storage.set(OWNER_ACCOUNT, Value::String(owner_account.to_string()))?;

        let txn = self.validator_config.build_transaction(
            0,
            self.fullnode_address,
            self.validator_address,
            false,
        )?;

        // Upload the validator config to shared storage
        let mut shared_storage = config.shared_backend();
        shared_storage.set(constants::VALIDATOR_CONFIG, Value::Transaction(txn.clone()))?;

        Ok(txn)
    }
}
