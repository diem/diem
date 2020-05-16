// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{error::Error, SecureBackends};
use libra_crypto::ed25519::Ed25519PublicKey;
use libra_secure_storage::{Storage, Value};
use std::convert::TryInto;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct AssociationKey {
    #[structopt(flatten)]
    pub secure_backends: SecureBackends,
}

impl AssociationKey {
    pub fn execute(self) -> Result<Ed25519PublicKey, Error> {
        submit_key(
            libra_global_constants::ASSOCIATION_KEY,
            self.secure_backends,
        )
    }
}

#[derive(Debug, StructOpt)]
pub struct OperatorKey {
    #[structopt(flatten)]
    pub secure_backends: SecureBackends,
}

impl OperatorKey {
    pub fn execute(self) -> Result<Ed25519PublicKey, Error> {
        submit_key(libra_global_constants::OPERATOR_KEY, self.secure_backends)
    }
}

#[derive(Debug, StructOpt)]
pub struct OwnerKey {
    #[structopt(flatten)]
    pub secure_backends: SecureBackends,
}

impl OwnerKey {
    pub fn execute(self) -> Result<Ed25519PublicKey, Error> {
        submit_key(libra_global_constants::OWNER_KEY, self.secure_backends)
    }
}

fn submit_key(key_name: &str, secure_backends: SecureBackends) -> Result<Ed25519PublicKey, Error> {
    let local: Box<dyn Storage> = secure_backends.local.try_into()?;
    if !local.available() {
        return Err(Error::LocalStorageUnavailable);
    }

    let key = local
        .get_public_key(key_name)
        .map_err(|e| Error::LocalStorageReadError(e.to_string()))?
        .public_key;

    if let Some(remote) = secure_backends.remote {
        let key = Value::Ed25519PublicKey(key.clone());
        let mut remote: Box<dyn Storage> = remote.try_into()?;
        if !remote.available() {
            return Err(Error::RemoteStorageUnavailable);
        }

        remote
            .set(key_name, key)
            .map_err(|e| Error::RemoteStorageWriteError(e.to_string()))?;
    }

    Ok(key)
}
