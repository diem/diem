// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_global_constants::OPERATOR_KEY;
use libra_management::{
    config::ConfigPath, constants, error::Error, secure_backend::SharedBackend,
};
use libra_secure_storage::Value;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct ValidatorOperator {
    #[structopt(flatten)]
    config: ConfigPath,
    #[structopt(long)]
    operator_name: String,
    #[structopt(flatten)]
    shared_backend: SharedBackend,
}

impl ValidatorOperator {
    pub fn execute(self) -> Result<String, Error> {
        let config = self
            .config
            .load()?
            .override_shared_backend(&self.shared_backend.shared_backend)?;
        let operator_name = self.operator_name;

        // Verify the operator exists in the shared storage
        let operator_storage = config.shared_backend_with_namespace(operator_name.clone());
        let _ = operator_storage.ed25519_key(OPERATOR_KEY)?;

        // Upload the operator name to shared storage
        let mut shared_storage = config.shared_backend();
        shared_storage.set(
            constants::VALIDATOR_OPERATOR,
            Value::String(operator_name.clone()),
        )?;

        Ok(operator_name)
    }
}
