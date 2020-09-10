// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{json_rpc::JsonRpcClientWrapper, TransactionContext};
use libra_global_constants::LIBRA_ROOT_KEY;
use libra_management::{config::Config, error::Error, transaction::build_raw_transaction};
use libra_types::{account_address::AccountAddress, account_config::libra_root_address};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct RootValidatorOperation {
    #[structopt(long, help = "The validator address")]
    account_address: AccountAddress,
    /// JSON-RPC Endpoint (e.g. http://localhost:8080)
    #[structopt(long, required_unless = "config")]
    json_server: Option<String>,
    #[structopt(flatten)]
    validator_config: libra_management::validator_config::ValidatorConfig,
}

impl RootValidatorOperation {
    fn config(&self) -> Result<Config, Error> {
        Ok(self
            .validator_config
            .config()?
            .override_json_server(&self.json_server))
    }
}

#[derive(Debug, StructOpt)]
pub struct AddValidator {
    #[structopt(flatten)]
    input: RootValidatorOperation,
}

impl AddValidator {
    pub fn execute(self) -> Result<TransactionContext, Error> {
        let config = self.input.config()?;
        let client = JsonRpcClientWrapper::new(config.json_server.clone());

        // Verify that this is a configured validator
        client.validator_config(self.input.account_address)?;
        let name = client
            .validator_config(self.input.account_address)?
            .human_name;

        // Prepare a transaction to add them to the ValidatorSet
        let seq_num = client.sequence_number(libra_root_address())?;
        let txn = build_raw_transaction(
            config.chain_id,
            libra_root_address(),
            seq_num,
            transaction_builder::encode_add_validator_and_reconfigure_script(
                seq_num,
                name,
                self.input.account_address,
            ),
        );

        let mut storage = config.validator_backend();
        let signed_txn = storage.sign(LIBRA_ROOT_KEY, "add-validator", txn)?;
        client.submit_transaction(signed_txn)
    }
}

#[derive(Debug, StructOpt)]
pub struct RemoveValidator {
    #[structopt(flatten)]
    input: RootValidatorOperation,
}

impl RemoveValidator {
    pub fn execute(self) -> Result<TransactionContext, Error> {
        let config = self.input.config()?;
        let client = JsonRpcClientWrapper::new(config.json_server.clone());

        // Verify that this is a validator within the set
        client.validator_set(Some(self.input.account_address))?;
        let name = client
            .validator_config(self.input.account_address)?
            .human_name;

        // Prepare a transaction to remove them from the ValidatorSet
        let seq_num = client.sequence_number(libra_root_address())?;
        let txn = build_raw_transaction(
            config.chain_id,
            libra_root_address(),
            seq_num,
            transaction_builder::encode_remove_validator_and_reconfigure_script(
                seq_num,
                name,
                self.input.account_address,
            ),
        );

        let mut storage = config.validator_backend();
        let signed_txn = storage.sign(LIBRA_ROOT_KEY, "remove-validator", txn)?;
        client.submit_transaction(signed_txn)
    }
}
