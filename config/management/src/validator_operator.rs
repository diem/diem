// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    constants,
    error::Error,
    secure_backend::StorageLocation::{LocalStorage, RemoteStorage},
    SecureBackends,
};
use libra_crypto::hash::CryptoHash;
use libra_global_constants::OWNER_KEY;
use libra_secure_storage::{CryptoStorage, KVStorage, Value};
use libra_secure_time::{RealTimeService, TimeService};
use libra_types::{
    account_address,
    transaction::{RawTransaction, SignedTransaction, Transaction},
};
use std::time::Duration;
use structopt::StructOpt;
use vm_genesis::get_account_address_from_name;

#[derive(Debug, StructOpt)]
pub struct ValidatorOperator {
    #[structopt(long)]
    operator_name: String,
    #[structopt(flatten)]
    backends: SecureBackends,
}

impl ValidatorOperator {
    pub fn execute(self) -> Result<Transaction, Error> {
        // Fetch the owner public key from local storage
        let mut local_storage = self.backends.local.create_storage(LocalStorage)?;
        let owner_key = local_storage
            .get_public_key(OWNER_KEY)
            .map_err(|e| Error::LocalStorageReadError(OWNER_KEY, e.to_string()))?
            .public_key;

        // Create the transaction script that sets the validator operator for the owner
        let operator_account = get_account_address_from_name(&self.operator_name);
        let set_operator_script =
            transaction_builder::encode_set_validator_operator_script(operator_account);

        // If a remote backend is specified, fetch the owner name from the shared repository
        // and use to derive an account address. Otherwise, derive the address from the public key.
        // TODO(joshlind): see if there's a better way to get access to the owner account here!
        let owner_account = if let Some(remote_config) = self.backends.remote.clone() {
            let owner_name = remote_config.get_namespace()?;
            get_account_address_from_name(&owner_name)
        } else {
            account_address::from_public_key(&owner_key)
        };

        // Create and sign the set operator transaction
        // TODO(joshlind): In genesis the sequence number is irrelevant. After genesis we need to
        // obtain the current sequence number by querying the blockchain.
        let sequence_number = 0;
        let expiration_time = RealTimeService::new().now() + constants::TXN_EXPIRATION_SECS;
        let raw_transaction = RawTransaction::new_script(
            owner_account,
            sequence_number,
            set_operator_script,
            constants::MAX_GAS_AMOUNT,
            constants::GAS_UNIT_PRICE,
            constants::GAS_CURRENCY_CODE.to_owned(),
            Duration::from_secs(expiration_time),
        );
        let signature = local_storage
            .sign_message(OWNER_KEY, &raw_transaction.hash())
            .map_err(|e| {
                Error::LocalStorageSigningError("set-operator", OWNER_KEY, e.to_string())
            })?;
        let signed_txn = SignedTransaction::new(raw_transaction, owner_key, signature);
        let signed_txn = Transaction::UserTransaction(signed_txn);

        // Upload the set operator transaction to shared storage
        if let Some(remote_config) = self.backends.remote {
            let mut remote_storage = remote_config.create_storage(RemoteStorage)?;
            remote_storage
                .set(
                    constants::VALIDATOR_OPERATOR,
                    Value::Transaction(signed_txn.clone()),
                )
                .map_err(|e| {
                    Error::RemoteStorageWriteError(constants::VALIDATOR_OPERATOR, e.to_string())
                })?;
        }

        Ok(signed_txn)
    }
}
