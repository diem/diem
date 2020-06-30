// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    error::Error,
    secure_backend::StorageLocation::{LocalStorage, RemoteStorage},
    SecureBackends,
};
use libra_crypto::ed25519::Ed25519PublicKey;
use libra_secure_storage::{CryptoStorage, KVStorage, Value};
use structopt::StructOpt;
use vm_genesis::get_account_address_from_name;

#[derive(Debug, StructOpt)]
pub struct AssociationKey {
    #[structopt(flatten)]
    pub secure_backends: SecureBackends,
}

impl AssociationKey {
    pub fn execute(self) -> Result<Ed25519PublicKey, Error> {
        submit_key(
            libra_global_constants::ASSOCIATION_KEY,
            None,
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
        submit_key(
            libra_global_constants::OPERATOR_KEY,
            Some(libra_global_constants::OPERATOR_ACCOUNT),
            self.secure_backends,
        )
    }
}

#[derive(Debug, StructOpt)]
pub struct OwnerKey {
    #[structopt(flatten)]
    pub secure_backends: SecureBackends,
}

impl OwnerKey {
    pub fn execute(self) -> Result<Ed25519PublicKey, Error> {
        submit_key(
            libra_global_constants::OWNER_KEY,
            Some(libra_global_constants::OWNER_ACCOUNT),
            self.secure_backends,
        )
    }
}

fn submit_key(
    key_name: &'static str,
    account_name: Option<&'static str>,
    secure_backends: SecureBackends,
) -> Result<Ed25519PublicKey, Error> {
    let mut local_storage = secure_backends.local.create_storage(LocalStorage)?;

    let key = local_storage
        .get_public_key(key_name)
        .map_err(|e| Error::LocalStorageReadError(key_name, e.to_string()))?
        .public_key;

    if let Some(account_name) = account_name {
        // If a remote backend is specified, fetch the owner/operator name from the shared repository
        // and use to derive an account address. Otherwise, derive the address from the public key.
        // TODO(joshlind): see if there's a better way to get access to the owner/operator account here!
        let account = if let Some(remote_config) = secure_backends.remote.clone() {
            let name = remote_config.get_namespace()?;
            let (_, account) = get_account_address_from_name(&name);
            account
        } else {
            libra_types::account_address::from_public_key(&key)
        };

        local_storage
            .set(account_name, Value::String(account.to_string()))
            .map_err(|e| Error::LocalStorageWriteError(account_name, e.to_string()))?
    }

    if let Some(remote) = secure_backends.remote {
        let mut remote_storage = remote.create_storage(RemoteStorage)?;
        remote_storage
            .set(key_name, Value::Ed25519PublicKey(key.clone()))
            .map_err(|e| Error::RemoteStorageWriteError(key_name, e.to_string()))?;
    }

    Ok(key)
}
