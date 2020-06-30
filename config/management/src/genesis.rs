// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    constants, error::Error, layout::Layout, secure_backend::StorageLocation::RemoteStorage,
    SingleBackend,
};
use libra_crypto::ed25519::Ed25519PublicKey;
use libra_global_constants::{ASSOCIATION_KEY, OPERATOR_KEY};
use libra_secure_storage::KVStorage;
use libra_types::transaction::{Transaction, TransactionPayload};
use std::{fs::File, io::Write, path::PathBuf};
use structopt::StructOpt;
use vm_genesis::ValidatorRegistration;

// TODO(davidiw) add operator_address, since that will eventually be the identity producing this.
/// Note, it is implicitly expected that the storage supports
/// a namespace but one has not been set.
#[derive(Debug, StructOpt)]
pub struct Genesis {
    #[structopt(flatten)]
    pub backend: SingleBackend,
    #[structopt(long)]
    pub path: Option<PathBuf>,
}

impl Genesis {
    pub fn execute(self) -> Result<Transaction, Error> {
        let layout = self.layout()?;
        let association_key = self.association(&layout)?;
        let validators = self.validators(&layout)?;

        let genesis = vm_genesis::encode_genesis_transaction_with_validator(
            association_key,
            &validators,
            Some(libra_types::on_chain_config::VMPublishingOption::Open),
        );

        if let Some(path) = self.path {
            let mut file = File::create(path).map_err(|e| {
                Error::UnexpectedError(format!("Unable to create genesis file: {}", e.to_string()))
            })?;
            let bytes = lcs::to_bytes(&genesis).map_err(|e| {
                Error::UnexpectedError(format!("Unable to serialize genesis: {}", e.to_string()))
            })?;
            file.write_all(&bytes).map_err(|e| {
                Error::UnexpectedError(format!("Unable to write genesis file: {}", e.to_string()))
            })?;
        }

        Ok(genesis)
    }

    /// Retrieves association key from the remote storage. Note, at this point in time, genesis
    /// only supports a single association key.
    pub fn association(&self, layout: &Layout) -> Result<Ed25519PublicKey, Error> {
        let association_config = self.backend.backend.clone();
        let association_config = association_config.set_namespace(layout.association[0].clone());

        let association_storage = association_config.create_storage(RemoteStorage)?;

        let association_key = association_storage
            .get(ASSOCIATION_KEY)
            .map_err(|e| Error::RemoteStorageReadError(ASSOCIATION_KEY, e.to_string()))?;
        association_key
            .value
            .ed25519_public_key()
            .map_err(|e| Error::RemoteStorageReadError(ASSOCIATION_KEY, e.to_string()))
    }

    /// Retrieves a layout from the remote storage.
    pub fn layout(&self) -> Result<Layout, Error> {
        let common_config = self.backend.backend.clone();
        let common_config = common_config.set_namespace(constants::COMMON_NS.into());

        let common_storage = common_config.create_storage(RemoteStorage)?;

        let layout = common_storage
            .get(constants::LAYOUT)
            .and_then(|v| v.value.string())
            .map_err(|e| Error::RemoteStorageReadError(constants::LAYOUT, e.to_string()))?;
        Layout::parse(&layout)
            .map_err(|e| Error::RemoteStorageReadError(constants::LAYOUT, e.to_string()))
    }

    /// Produces a set of ValidatorRegistration from the remote storage.
    pub fn validators(&self, layout: &Layout) -> Result<Vec<ValidatorRegistration>, Error> {
        let mut validators = Vec::new();
        for operator in layout.operators.iter() {
            let validator_config = self.backend.backend.clone();
            let validator_config = validator_config.set_namespace(operator.into());

            let validator_storage = validator_config.create_storage(RemoteStorage)?;

            let key = validator_storage
                .get(OPERATOR_KEY)
                .map_err(|e| Error::RemoteStorageReadError(OPERATOR_KEY, e.to_string()))?
                .value
                .ed25519_public_key()
                .map_err(|e| Error::RemoteStorageReadError(OPERATOR_KEY, e.to_string()))?;

            let txn = validator_storage
                .get(constants::VALIDATOR_CONFIG)
                .map_err(|e| {
                    Error::RemoteStorageReadError(constants::VALIDATOR_CONFIG, e.to_string())
                })?
                .value;
            let txn = txn.transaction().unwrap();
            let txn = txn.as_signed_user_txn().unwrap().payload();
            let txn = if let TransactionPayload::Script(script) = txn {
                script.clone()
            } else {
                return Err(Error::UnexpectedError("Found invalid registration".into()));
            };

            validators.push((key, txn));
        }

        Ok(validators)
    }
}
