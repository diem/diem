// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    constants,
    error::Error,
    secure_backend::StorageLocation::{LocalStorage, RemoteStorage},
    SecureBackends,
};
use libra_global_constants::{OPERATOR_KEY, OWNER_ACCOUNT, OWNER_KEY};
use libra_secure_storage::{CryptoStorage, KVStorage, Value};
use libra_secure_time::{RealTimeService, TimeService};
use libra_types::{
    account_address,
    account_address::AccountAddress,
    transaction::{RawTransaction, Script, SignedTransaction, Transaction},
};
use std::{str::FromStr, time::Duration};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct ValidatorOperator {
    #[structopt(long)]
    operator_name: String,
    #[structopt(flatten)]
    backends: SecureBackends,
}

impl ValidatorOperator {
    pub fn execute(self) -> Result<Transaction, Error> {
        // Fetch the operator key from remote storage using the operator_name and derive an address
        let operator_account = self.fetch_operator_account()?;

        // Create the transaction script that sets the validator operator for the owner
        let set_operator_script =
            transaction_builder::encode_set_validator_operator_script(operator_account);

        // Create and sign the set-operator transaction
        let set_operator_tx = self.create_set_operator_transaction(set_operator_script)?;

        // Upload the set-operator transaction to shared storage
        match self.backends.remote {
            None => return Err(Error::RemoteStorageMissing),
            Some(remote_config) => {
                let mut remote_storage = remote_config.create_storage(RemoteStorage)?;
                remote_storage
                    .set(
                        constants::VALIDATOR_OPERATOR,
                        Value::Transaction(set_operator_tx.clone()),
                    )
                    .map_err(|e| {
                        Error::RemoteStorageWriteError(constants::VALIDATOR_OPERATOR, e.to_string())
                    })?;
            }
        };

        Ok(set_operator_tx)
    }

    /// Creates and returns a signed set-operator transaction to be uploaded to the remote storage.
    fn create_set_operator_transaction(&self, script: Script) -> Result<Transaction, Error> {
        // Fetch the owner key and account address from local storage
        let local_storage = &mut self.backends.local.clone().create_storage(LocalStorage)?;
        let owner_key = local_storage
            .get_public_key(OWNER_KEY)
            .map_err(|e| Error::LocalStorageReadError(OWNER_KEY, e.to_string()))?
            .public_key;
        let owner_address_string = local_storage
            .get(OWNER_ACCOUNT)
            .and_then(|v| v.value.string())
            .map_err(|e| Error::LocalStorageReadError(OWNER_ACCOUNT, e.to_string()))?;
        let owner_address = AccountAddress::from_str(&owner_address_string)
            .map_err(|e| Error::BackendParsingError(e.to_string()))?;

        // Create the set-operator transaction using the owner_address as the sender
        // TODO(joshlind): In genesis the sequence number is irrelevant. After genesis we need to
        // obtain the current sequence number by querying the blockchain.
        let sequence_number = 0;
        let expiration_time = RealTimeService::new().now() + constants::TXN_EXPIRATION_SECS;
        let raw_transaction = RawTransaction::new_script(
            owner_address,
            sequence_number,
            script,
            constants::MAX_GAS_AMOUNT,
            constants::GAS_UNIT_PRICE,
            constants::GAS_CURRENCY_CODE.to_owned(),
            Duration::from_secs(expiration_time),
        );

        // Sign and return the set-operator transaction
        let signature = local_storage
            .sign(OWNER_KEY, &raw_transaction)
            .map_err(|e| {
                Error::LocalStorageSigningError("set-operator", OWNER_KEY, e.to_string())
            })?;
        let signed_txn = SignedTransaction::new(raw_transaction, owner_key, signature);
        Ok(Transaction::UserTransaction(signed_txn))
    }

    /// Retrieves the operator key from the remote storage using the operator name given by
    /// the set-operator command, and uses this key to derive an operator account address.
    /// If a remote storage path is not specified, returns an error.
    fn fetch_operator_account(&self) -> Result<AccountAddress, Error> {
        match self.backends.remote.clone() {
            None => Err(Error::RemoteStorageMissing),
            Some(operator_config) => {
                let operator_config = operator_config.set_namespace(self.operator_name.clone());
                let operator_storage = operator_config.create_storage(RemoteStorage)?;
                let operator_key = operator_storage
                    .get(OPERATOR_KEY)
                    .map_err(|e| Error::RemoteStorageReadError(OPERATOR_KEY, e.to_string()))?
                    .value
                    .ed25519_public_key()
                    .map_err(|e| Error::RemoteStorageReadError(OPERATOR_KEY, e.to_string()))?;
                Ok(account_address::from_public_key(&operator_key))
            }
        }
    }
}
