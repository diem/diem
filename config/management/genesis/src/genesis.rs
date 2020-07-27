// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::layout::Layout;
use libra_crypto::ed25519::Ed25519PublicKey;
use libra_global_constants::{LIBRA_ROOT_KEY, OPERATOR_KEY, OWNER_KEY};
use libra_management::{
    config::ConfigPath, constants, error::Error, secure_backend::SharedBackend,
};
use libra_types::{
    account_address,
    chain_id::ChainId,
    transaction::{Transaction, TransactionPayload},
};
use std::{fs::File, io::Write, path::PathBuf};
use structopt::StructOpt;
use vm_genesis::{OperatorAssignment, OperatorRegistration};

/// Note, it is implicitly expected that the storage supports
/// a namespace but one has not been set.
#[derive(Debug, StructOpt)]
pub struct Genesis {
    #[structopt(flatten)]
    pub config: ConfigPath,
    #[structopt(flatten)]
    pub backend: SharedBackend,
    #[structopt(long)]
    pub path: Option<PathBuf>,
}

impl Genesis {
    pub fn execute(self) -> Result<Transaction, Error> {
        let layout = self.layout()?;
        let libra_root_key = self.libra_root_key(&layout)?;
        let operator_assignments = self.operator_assignments(&layout)?;
        let operator_registrations = self.operator_registrations(&layout)?;

        let genesis = vm_genesis::encode_genesis_transaction(
            libra_root_key,
            &operator_assignments,
            &operator_registrations,
            Some(libra_types::on_chain_config::VMPublishingOption::open()), // TODO: intended?
            ChainId::test(), // TODO: where should we read this from?
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

    /// Retrieves the libra root key from the remote storage. Note, at this point in time, genesis
    /// only supports a single libra root key.
    pub fn libra_root_key(&self, layout: &Layout) -> Result<Ed25519PublicKey, Error> {
        let config = self
            .config
            .load()?
            .override_shared_backend(&self.backend.shared_backend)?;
        let storage = config.shared_backend_with_namespace(layout.libra_root[0].clone());
        storage.ed25519_key(LIBRA_ROOT_KEY)
    }

    /// Retrieves a layout from the remote storage.
    pub fn layout(&self) -> Result<Layout, Error> {
        let config = self
            .config
            .load()?
            .override_shared_backend(&self.backend.shared_backend)?;
        let storage = config.shared_backend_with_namespace(constants::COMMON_NS.into());

        let layout = storage.string(constants::LAYOUT)?;
        Layout::parse(&layout).map_err(|e| Error::UnableToParse(constants::LAYOUT, e.to_string()))
    }

    /// Produces a set of OperatorAssignments from the remote storage.
    pub fn operator_assignments(&self, layout: &Layout) -> Result<Vec<OperatorAssignment>, Error> {
        let config = self
            .config
            .load()?
            .override_shared_backend(&self.backend.shared_backend)?;
        let mut operator_assignments = Vec::new();

        for owner in layout.owners.iter() {
            let owner_storage = config.shared_backend_with_namespace(owner.into());
            let owner_key = owner_storage.ed25519_key(OWNER_KEY)?;

            let operator_name = owner_storage.string(constants::VALIDATOR_OPERATOR)?;
            let operator_storage = config.shared_backend_with_namespace(operator_name);
            let operator_key = operator_storage.ed25519_key(OPERATOR_KEY)?;
            let operator_account = account_address::from_public_key(&operator_key);

            let set_operator_script =
                transaction_builder::encode_set_validator_operator_script(operator_account);

            operator_assignments.push((owner_key, set_operator_script));
        }

        Ok(operator_assignments)
    }

    /// Produces a set of OperatorRegistrations from the remote storage.
    pub fn operator_registrations(
        &self,
        layout: &Layout,
    ) -> Result<Vec<OperatorRegistration>, Error> {
        let config = self
            .config
            .load()?
            .override_shared_backend(&self.backend.shared_backend)?;
        let mut registrations = Vec::new();

        for operator in layout.operators.iter() {
            let operator_storage = config.shared_backend_with_namespace(operator.into());
            let operator_key = operator_storage.ed25519_key(OPERATOR_KEY)?;
            let validator_config_tx = operator_storage.transaction(constants::VALIDATOR_CONFIG)?;
            let validator_config_tx = validator_config_tx.as_signed_user_txn().unwrap().payload();
            let validator_config_tx =
                if let TransactionPayload::Script(script) = validator_config_tx {
                    script.clone()
                } else {
                    return Err(Error::UnexpectedError("Found invalid registration".into()));
                };

            registrations.push((operator_key, validator_config_tx));
        }

        Ok(registrations)
    }
}
