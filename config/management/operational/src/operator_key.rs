// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{json_rpc::JsonRpcClientWrapper, TransactionContext};
use libra_crypto::ed25519::Ed25519PublicKey;
use libra_global_constants::{OPERATOR_ACCOUNT, OPERATOR_KEY};
use libra_management::{error::Error, transaction::build_raw_transaction};
use libra_types::transaction::{authenticator::AuthenticationKey, Transaction};
use std::convert::TryFrom;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct RotateOperatorKey {
    /// JSON-RPC Endpoint (e.g. http://localhost:8080)
    #[structopt(long, required_unless = "config")]
    json_server: Option<String>,
    #[structopt(flatten)]
    validator_config: libra_management::validator_config::ValidatorConfig,
}

impl RotateOperatorKey {
    pub fn execute(self) -> Result<(TransactionContext, Ed25519PublicKey), Error> {
        // Load the config, storage backend and create a json rpc client
        let config = self
            .validator_config
            .config()?
            .override_json_server(&self.json_server);
        let mut storage = config.validator_backend();
        let client = JsonRpcClientWrapper::new(config.json_server);

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
        let rotate_key_script = transaction_builder::encode_rotate_authentication_key_script(
            AuthenticationKey::ed25519(&new_storage_key).to_vec(),
        );
        let rotate_key_txn = build_raw_transaction(
            config.chain_id,
            operator_account,
            sequence_number,
            rotate_key_script,
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
        let txn_ctx =
            client.submit_transaction(rotate_key_txn.as_signed_user_txn().unwrap().clone())?;
        Ok((txn_ctx, new_storage_key))
    }
}
