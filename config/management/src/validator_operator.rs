// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{constants, error::Error, SingleBackend};
use libra_crypto::ed25519::Ed25519PublicKey;
use libra_secure_storage::{KVStorage, Storage, Value};
use std::convert::TryInto;
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

        let mut remote: Storage = self.backend.backend.try_into()?;
        remote
            .available()
            .map_err(|e| Error::RemoteStorageUnavailable(e.to_string()))?;
        remote
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
        let mut operator_config = self.backend.backend.clone();
        operator_config
            .parameters
            .insert("namespace".into(), self.operator_name.clone());

        let operator_storage: Storage = operator_config.try_into()?;
        operator_storage
            .available()
            .map_err(|e| Error::RemoteStorageUnavailable(e.to_string()))?;

        operator_storage
            .get(libra_global_constants::OPERATOR_KEY)
            .and_then(|v| v.value.ed25519_public_key())
            .map_err(|e| {
                Error::RemoteStorageReadError(libra_global_constants::OPERATOR_KEY, e.to_string())
            })
    }
}
