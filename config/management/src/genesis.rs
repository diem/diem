// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{constants, error::Error, layout::Layout, SingleBackend};
use libra_crypto::ed25519::Ed25519PublicKey;
use libra_global_constants::{ASSOCIATION_KEY, OPERATOR_KEY};
use libra_secure_storage::{KVStorage, Storage};
use libra_types::transaction::{Transaction, TransactionPayload};
use std::{convert::TryInto, fs::File, io::Write, path::PathBuf};
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
            None,
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
        let mut association_config = self.backend.backend.clone();
        association_config
            .parameters
            .insert("namespace".into(), layout.association[0].clone());

        let association: Storage = association_config.try_into()?;
        association
            .available()
            .map_err(|e| Error::RemoteStorageUnavailable(e.to_string()))?;

        let association_key = association
            .get(ASSOCIATION_KEY)
            .map_err(|e| Error::RemoteStorageReadError(ASSOCIATION_KEY, e.to_string()))?;
        association_key
            .value
            .ed25519_public_key()
            .map_err(|e| Error::RemoteStorageReadError(ASSOCIATION_KEY, e.to_string()))
    }

    /// Retrieves a layout from the remote storage.
    pub fn layout(&self) -> Result<Layout, Error> {
        let mut common_config = self.backend.backend.clone();
        common_config
            .parameters
            .insert("namespace".into(), constants::COMMON_NS.into());

        let common: Storage = common_config.try_into()?;
        common
            .available()
            .map_err(|e| Error::RemoteStorageUnavailable(e.to_string()))?;

        let layout = common
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
            let mut validator_config = self.backend.backend.clone();
            validator_config
                .parameters
                .insert("namespace".into(), operator.into());

            let validator: Storage = validator_config.try_into()?;
            validator
                .available()
                .map_err(|e| Error::RemoteStorageUnavailable(e.to_string()))?;

            let key = validator
                .get(OPERATOR_KEY)
                .map_err(|e| Error::RemoteStorageReadError(OPERATOR_KEY, e.to_string()))?
                .value
                .ed25519_public_key()
                .map_err(|e| Error::RemoteStorageReadError(OPERATOR_KEY, e.to_string()))?;

            let txn = validator
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
