// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{auto_validate::AutoValidate, json_rpc::JsonRpcClientWrapper, TransactionContext};
use diem_crypto::ed25519::Ed25519PublicKey;
use diem_global_constants::{OPERATOR_ACCOUNT, OPERATOR_KEY};
use diem_management::{error::Error, transaction::build_raw_transaction};
use diem_transaction_builder::stdlib as transaction_builder;
use diem_types::{
    account_address::AccountAddress,
    transaction::{authenticator::AuthenticationKey, Transaction},
};
use serde::Serialize;
use std::convert::TryFrom;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct AccountResource {
    #[structopt(long, help = "Account address to display the account resource")]
    account_address: AccountAddress,
    #[structopt(flatten)]
    config: diem_management::config::ConfigPath,
    /// JSON-RPC Endpoint (e.g. http://localhost:8080)
    #[structopt(long, required_unless = "config")]
    json_server: Option<String>,
}

impl AccountResource {
    pub fn execute(self) -> Result<SimplifiedAccountResource, Error> {
        // Load the config and create a json rpc client
        let config = self.config.load()?.override_json_server(&self.json_server);
        let client = JsonRpcClientWrapper::new(config.json_server);

        // Fetch the current account resource on-chain for the specified account address.
        let account_resource = client.account_resource(self.account_address)?;
        Ok(SimplifiedAccountResource {
            account: self.account_address,
            authentication_key: hex::encode(account_resource.authentication_key()),
            sequence_number: account_resource.sequence_number(),
        })
    }
}

/// Used to display a subset version of information from an account resource fetched for a
/// specific account.
/// Note: the authentication is hex encoded.
#[derive(Serialize)]
pub struct SimplifiedAccountResource {
    pub account: AccountAddress,
    pub authentication_key: String,
    pub sequence_number: u64,
}

#[derive(Debug, StructOpt)]
pub struct RotateOperatorKey {
    /// JSON-RPC Endpoint (e.g. http://localhost:8080)
    #[structopt(long, required_unless = "config")]
    json_server: Option<String>,
    #[structopt(flatten)]
    validator_config: diem_management::validator_config::ValidatorConfig,
    #[structopt(flatten)]
    auto_validate: AutoValidate,
}

impl RotateOperatorKey {
    pub fn execute(self) -> Result<(TransactionContext, Ed25519PublicKey), Error> {
        // Load the config, storage backend and create a json rpc client
        let config = self
            .validator_config
            .config()?
            .override_json_server(&self.json_server);
        let mut storage = config.validator_backend();
        let client = JsonRpcClientWrapper::new(config.json_server.clone());

        // Fetch the current on-chain auth key for the operator and the current key held in storage.
        let operator_account = storage.account_address(OPERATOR_ACCOUNT)?;
        let account_resource = client.account_resource(operator_account)?;
        let on_chain_key = match AuthenticationKey::try_from(account_resource.authentication_key())
        {
            Ok(auth_key) => auth_key,
            Err(e) => {
                return Err(Error::UnexpectedError(format!(
                    "Invalid authentication key found in account resource. Error: {}",
                    e.to_string()
                )));
            }
        };
        let mut current_storage_key = storage.ed25519_public_from_private(OPERATOR_KEY)?;

        // Check that the key held in storage matches the key registered on-chain in the validator
        // config. If so, rotate the key in storage. If not, fetch the previous key version from
        // storage so that we can allow the next step to resubmit the key rotation transaction
        // (to resynchronize storage with the blockchain).
        let new_storage_key = if on_chain_key == AuthenticationKey::ed25519(&current_storage_key) {
            storage.rotate_key(OPERATOR_KEY)?
        } else {
            let new_storage_key = current_storage_key;
            current_storage_key =
                storage.ed25519_public_from_private_previous_version(OPERATOR_KEY)?;
            new_storage_key
        };

        // Fetch the current sequence number
        let sequence_number = client.sequence_number(operator_account)?;

        // Build the operator rotation transaction
        let rotate_key_script =
            transaction_builder::encode_rotate_authentication_key_script_function(
                AuthenticationKey::ed25519(&new_storage_key).to_vec(),
            );
        let rotate_key_txn = build_raw_transaction(
            config.chain_id,
            operator_account,
            sequence_number,
            rotate_key_script.into_script_function(),
        );

        // Sign the operator rotation transaction
        let rotate_key_txn = storage.sign_using_version(
            OPERATOR_KEY,
            current_storage_key,
            "rotate-operator-key",
            rotate_key_txn,
        )?;
        let rotate_key_txn = Transaction::UserTransaction(rotate_key_txn);

        // Submit the transaction
        let mut transaction_context =
            client.submit_transaction(rotate_key_txn.as_signed_user_txn().unwrap().clone())?;

        // Perform auto validation if required
        transaction_context = self
            .auto_validate
            .execute(config.json_server, transaction_context)?;

        Ok((transaction_context, new_storage_key))
    }
}
