// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    constants, error::Error, secure_backend::StorageLocation::RemoteStorage, SecureBackends,
};
use libra_global_constants::OPERATOR_KEY;
use libra_secure_storage::{KVStorage, Value};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct ValidatorOperator {
    #[structopt(long)]
    operator_name: String,
    #[structopt(flatten)]
    backends: SecureBackends,
}

impl ValidatorOperator {
    pub fn execute(self) -> Result<String, Error> {
        // Get the operator name and verify it exists in the remote storage
        let operator_name = self.get_and_verify_operator_name()?;

        // Upload the operator name to shared storage
        match self.backends.remote {
            None => return Err(Error::RemoteStorageMissing),
            Some(remote_config) => {
                let mut remote_storage = remote_config.create_storage(RemoteStorage)?;
                remote_storage
                    .set(
                        constants::VALIDATOR_OPERATOR,
                        Value::String(operator_name.clone()),
                    )
                    .map_err(|e| {
                        Error::RemoteStorageWriteError(constants::VALIDATOR_OPERATOR, e.to_string())
                    })?;
            }
        };

        Ok(operator_name)
    }

    /// Verifies the operator name (given by the set-operator command) exists in remote storage.
    /// If the named operator is not found (i.e., the operator has not uploaded a public key) return
    /// an error. Otherwise, return the operator name.
    fn get_and_verify_operator_name(&self) -> Result<String, Error> {
        match self.backends.remote.clone() {
            None => Err(Error::RemoteStorageMissing),
            Some(operator_config) => {
                let operator_config = operator_config.set_namespace(self.operator_name.clone());
                let operator_storage = operator_config.create_storage(RemoteStorage)?;
                let _ = operator_storage
                    .get(OPERATOR_KEY)
                    .map_err(|e| Error::RemoteStorageReadError(OPERATOR_KEY, e.to_string()))?
                    .value
                    .ed25519_public_key()
                    .map_err(|e| Error::RemoteStorageReadError(OPERATOR_KEY, e.to_string()))?;
                Ok(self.operator_name.clone())
            }
        }
    }
}
