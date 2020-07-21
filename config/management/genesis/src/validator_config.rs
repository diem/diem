// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_global_constants::{OWNER_ACCOUNT, OWNER_KEY};
use libra_management::{
    constants, error::Error, secure_backend::SharedBackend, storage::StorageWrapper,
};
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
        // Retrieve and set owner account
        let owner_shared_storage = StorageWrapper::new_with_namespace(
            self.shared_backend.name(),
            self.owner_name,
            &self.shared_backend.shared_backend,
        )?;
        let owner_key = owner_shared_storage.ed25519_key(OWNER_KEY)?;
        let owner_account = account_address::from_public_key(&owner_key);

        let mut validator_storage = StorageWrapper::new(
            self.validator_config.validator_backend.name(),
            &self.validator_config.validator_backend.validator_backend,
        )?;
        validator_storage.set(OWNER_ACCOUNT, Value::String(owner_account.to_string()))?;

        let txn = self.validator_config.build_transaction(
            0,
            self.fullnode_address,
            self.validator_address,
            false,
        )?;

        // Upload the validator config to shared storage
        let mut shared_storage = StorageWrapper::new(
            self.shared_backend.name(),
            &self.shared_backend.shared_backend,
        )?;
        shared_storage.set(constants::VALIDATOR_CONFIG, Value::Transaction(txn.clone()))?;

        Ok(txn)
    }
}
