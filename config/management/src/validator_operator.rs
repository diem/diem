// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    constants, error::Error, secure_backend::StorageLocation::RemoteStorage, SingleBackend,
};
use libra_crypto::ed25519::Ed25519PublicKey;
use libra_secure_storage::{KVStorage, Value};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct ValidatorOperator {
    #[structopt(long)]
    operator_name: String,
    #[structopt(flatten)]
    backend: SingleBackend,
}

impl ValidatorOperator {
    pub fn execute(self) -> Result<Ed25519PublicKey, Error> {
        let operator_key = self.fetch_operator_key()?;

        let remote_config = self.backend.backend;
        let mut remote_storage =
            remote_config.new_available_storage_with_namespace(RemoteStorage, None)?;
        remote_storage
            .set(
                constants::VALIDATOR_OPERATOR,
                Value::Ed25519PublicKey(operator_key.clone()),
            )
            .map_err(|e| {
                Error::RemoteStorageWriteError(constants::VALIDATOR_OPERATOR, e.to_string())
            })?;

        Ok(operator_key)
    }

    /// Retrieves the operator key from the remote storage using the operator name given by
    /// the set-operator command.
    fn fetch_operator_key(&self) -> Result<Ed25519PublicKey, Error> {
        let operator_config = self.backend.backend.clone();
        let operator_storage = operator_config.new_available_storage_with_namespace(
            RemoteStorage,
            Some(self.operator_name.clone()),
        )?;

        operator_storage
            .get(libra_global_constants::OPERATOR_KEY)
            .and_then(|v| v.value.ed25519_public_key())
            .map_err(|e| {
                Error::RemoteStorageReadError(libra_global_constants::OPERATOR_KEY, e.to_string())
            })
    }
}
