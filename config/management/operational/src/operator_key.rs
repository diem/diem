// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{json_rpc::JsonRpcClientWrapper, validator_config::RotateKey, TransactionContext};
use libra_crypto::ed25519::Ed25519PublicKey;
use libra_global_constants::{OPERATOR_ACCOUNT, OPERATOR_KEY};
use libra_management::{constants, error::Error, storage::StorageWrapper};
use libra_secure_time::{RealTimeService, TimeService};
use libra_types::transaction::{authenticator::AuthenticationKey, RawTransaction, Transaction};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct RotateOperatorKey {
    #[structopt(flatten)]
    rotate_key: RotateKey,
}

impl RotateOperatorKey {
    pub fn execute(self) -> Result<(TransactionContext, Ed25519PublicKey), Error> {
        // Fetch the operator account from storage
        let mut storage = StorageWrapper::new(
            &self.rotate_key.validator_config.validator_backend.name(),
            &self
                .rotate_key
                .validator_config
                .validator_backend
                .validator_backend,
        )?;
        let operator_account = storage.account_address(OPERATOR_ACCOUNT)?;

        // Create a JSON RPC client and fetch the current sequence number
        let client = JsonRpcClientWrapper::new(self.rotate_key.host);
        let sequence_number = client.sequence_number(operator_account)?;

        // Rotate the operator key in storage
        let current_operator_key = storage.ed25519_public_from_private(OPERATOR_KEY)?;
        let new_operator_key = storage.rotate_key(OPERATOR_KEY)?;

        // Build the operator rotation transaction
        let rotate_key_script = transaction_builder::encode_rotate_authentication_key_script(
            AuthenticationKey::ed25519(&new_operator_key).to_vec(),
        );
        let rotate_key_txn = RawTransaction::new_script(
            operator_account,
            sequence_number,
            rotate_key_script,
            constants::MAX_GAS_AMOUNT,
            constants::GAS_UNIT_PRICE,
            constants::GAS_CURRENCY_CODE.to_owned(),
            RealTimeService::new().now() + constants::TXN_EXPIRATION_SECS,
            self.rotate_key.validator_config.chain_id,
        );

        // Sign the operator rotation transaction
        let rotate_key_txn = storage.sign_using_version(
            OPERATOR_KEY,
            current_operator_key,
            "rotate-operator-key",
            rotate_key_txn,
        )?;
        let rotate_key_txn = Transaction::UserTransaction(rotate_key_txn);

        // Submit the transaction
        let txn_ctx =
            client.submit_transaction(rotate_key_txn.as_signed_user_txn().unwrap().clone())?;
        Ok((txn_ctx, new_operator_key))
    }
}
