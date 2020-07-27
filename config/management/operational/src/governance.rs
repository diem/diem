// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{json_rpc::JsonRpcClientWrapper, TransactionContext};
use libra_global_constants::LIBRA_ROOT_KEY;
use libra_management::{
    config::{Config, ConfigPath},
    constants,
    error::Error,
    secure_backend::ValidatorBackend,
};
use libra_secure_time::{RealTimeService, TimeService};
use libra_types::{
    account_address::AccountAddress, account_config::libra_root_address, chain_id::ChainId,
    transaction::RawTransaction,
};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct RootValidatorOperation {
    #[structopt(flatten)]
    pub config: ConfigPath,
    #[structopt(long, help = "The validator address to be added")]
    account_address: AccountAddress,
    #[structopt(flatten)]
    backend: ValidatorBackend,
    #[structopt(long, required_unless = "config")]
    chain_id: Option<ChainId>,
    /// JSON-RPC Endpoint (e.g. http://localhost:8080)
    #[structopt(long, required_unless = "config")]
    json_server: Option<String>,
}

impl RootValidatorOperation {
    fn config(&self) -> Result<Config, Error> {
        self.config
            .load()?
            .override_chain_id(self.chain_id)
            .override_json_server(&self.json_server)
            .override_validator_backend(&self.backend.validator_backend)
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

        // Prepare a transaction to add them to the ValidatorSet
        let seq_num = client.sequence_number(libra_root_address())?;
        let txn = RawTransaction::new_script(
            libra_root_address(),
            seq_num,
            transaction_builder::encode_add_validator_script(self.input.account_address),
            constants::MAX_GAS_AMOUNT,
            constants::GAS_UNIT_PRICE,
            constants::GAS_CURRENCY_CODE.to_owned(),
            RealTimeService::new().now() + constants::TXN_EXPIRATION_SECS,
            config.chain_id,
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

        // Prepare a transaction to remove them from the ValidatorSet
        let seq_num = client.sequence_number(libra_root_address())?;
        let txn = RawTransaction::new_script(
            libra_root_address(),
            seq_num,
            transaction_builder::encode_remove_validator_script(self.input.account_address),
            constants::MAX_GAS_AMOUNT,
            constants::GAS_UNIT_PRICE,
            constants::GAS_CURRENCY_CODE.to_owned(),
            RealTimeService::new().now() + constants::TXN_EXPIRATION_SECS,
            config.chain_id,
        );

        let mut storage = config.validator_backend();
        let signed_txn = storage.sign(LIBRA_ROOT_KEY, "remove-validator", txn)?;
        client.submit_transaction(signed_txn)
    }
}
