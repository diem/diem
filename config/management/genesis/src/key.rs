// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_crypto::ed25519::Ed25519PublicKey;
use libra_management::{
    error::Error,
    secure_backend::{OptionalSharedBackend, ValidatorBackend},
};
use libra_secure_storage::{CryptoStorage, KVStorage, Value};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct LibraRootKey {
    #[structopt(flatten)]
    validator_backend: ValidatorBackend,
    #[structopt(flatten)]
    shared_backend: OptionalSharedBackend,
}

impl LibraRootKey {
    pub fn execute(self) -> Result<Ed25519PublicKey, Error> {
        submit_key(
            libra_global_constants::LIBRA_ROOT_KEY,
            None,
            self.validator_backend,
            self.shared_backend,
        )
    }
}

#[derive(Debug, StructOpt)]
pub struct OperatorKey {
    #[structopt(flatten)]
    validator_backend: ValidatorBackend,
    #[structopt(flatten)]
    shared_backend: OptionalSharedBackend,
}

impl OperatorKey {
    pub fn execute(self) -> Result<Ed25519PublicKey, Error> {
        submit_key(
            libra_global_constants::OPERATOR_KEY,
            Some(libra_global_constants::OPERATOR_ACCOUNT),
            self.validator_backend,
            self.shared_backend,
        )
    }
}

#[derive(Debug, StructOpt)]
pub struct OwnerKey {
    #[structopt(flatten)]
    validator_backend: ValidatorBackend,
    #[structopt(flatten)]
    shared_backend: OptionalSharedBackend,
}

impl OwnerKey {
    pub fn execute(self) -> Result<Ed25519PublicKey, Error> {
        submit_key(
            libra_global_constants::OWNER_KEY,
            Some(libra_global_constants::OWNER_ACCOUNT),
            self.validator_backend,
            self.shared_backend,
        )
    }
}

fn submit_key(
    key_name: &'static str,
    account_name: Option<&'static str>,
    validator_backend: ValidatorBackend,
    shared_backend: OptionalSharedBackend,
) -> Result<Ed25519PublicKey, Error> {
    let mut validator_storage = validator_backend
        .validator_backend
        .create_storage(validator_backend.name())?;

    let key = validator_storage
        .get_public_key(key_name)
        .map_err(|e| Error::StorageReadError(validator_backend.name(), key_name, e.to_string()))?
        .public_key;

    if let Some(account_name) = account_name {
        let peer_id = libra_types::account_address::from_public_key(&key);
        validator_storage
            .set(account_name, Value::String(peer_id.to_string()))
            .map_err(|e| {
                Error::StorageWriteError(validator_backend.name(), account_name, e.to_string())
            })?
    }

    if let Some(shared_backend_config) = &shared_backend.shared_backend {
        let mut shared_storage = shared_backend_config.create_storage(shared_backend.name())?;
        shared_storage
            .set(key_name, Value::Ed25519PublicKey(key.clone()))
            .map_err(|e| {
                Error::StorageWriteError(shared_backend.name(), key_name, e.to_string())
            })?;
    }

    Ok(key)
}
