// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{json_rpc::JsonRpcClientWrapper, TransactionContext};
use libra_global_constants::LIBRA_ROOT_KEY;
use libra_management::{
    constants, error::Error, secure_backend::ValidatorBackend, storage::StorageWrapper,
};
use libra_secure_time::{RealTimeService, TimeService};
use libra_types::{
    account_address::AccountAddress, account_config::libra_root_address, chain_id::ChainId,
    transaction::RawTransaction,
};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct AddValidator {
    #[structopt(long, help = "JSON-RPC Endpoint (e.g. http://localhost:8080")]
    host: String,
    #[structopt(long)]
    chain_id: ChainId,
    #[structopt(long, help = "The validator address to be added")]
    validator_address: AccountAddress,
    #[structopt(flatten)]
    libra_root: ValidatorBackend,
}

impl AddValidator {
    pub fn execute(self) -> Result<TransactionContext, Error> {
        let client = JsonRpcClientWrapper::new(self.host);
        client.validator_config(self.validator_address)?;
        let seq_num = client.sequence_number(libra_root_address())?;
        let txn = RawTransaction::new_script(
            libra_root_address(),
            seq_num,
            transaction_builder::encode_add_validator_script(self.validator_address),
            constants::MAX_GAS_AMOUNT,
            constants::GAS_UNIT_PRICE,
            constants::GAS_CURRENCY_CODE.to_owned(),
            RealTimeService::new().now() + constants::TXN_EXPIRATION_SECS,
            self.chain_id,
        );
        let mut storage =
            StorageWrapper::new(self.libra_root.name(), &self.libra_root.validator_backend)?;
        let signed_txn = storage.sign(LIBRA_ROOT_KEY, "add-validator", txn)?;
        client.submit_transaction(signed_txn)
    }
}

#[derive(Debug, StructOpt)]
pub struct RemoveValidator {
    #[structopt(long, help = "JSON-RPC Endpoint (e.g. http://localhost:8080")]
    host: String,
    #[structopt(long)]
    chain_id: ChainId,
    #[structopt(long, help = "The validator address to be removed")]
    validator_address: AccountAddress,
    #[structopt(flatten)]
    libra_root: ValidatorBackend,
}

impl RemoveValidator {
    pub fn execute(self) -> Result<TransactionContext, Error> {
        let client = JsonRpcClientWrapper::new(self.host);
        client.validator_set(Some(self.validator_address))?;
        let seq_num = client.sequence_number(libra_root_address())?;
        let txn = RawTransaction::new_script(
            libra_root_address(),
            seq_num,
            transaction_builder::encode_remove_validator_script(self.validator_address),
            constants::MAX_GAS_AMOUNT,
            constants::GAS_UNIT_PRICE,
            constants::GAS_CURRENCY_CODE.to_owned(),
            RealTimeService::new().now() + constants::TXN_EXPIRATION_SECS,
            self.chain_id,
        );
        let mut storage =
            StorageWrapper::new(self.libra_root.name(), &self.libra_root.validator_backend)?;
        let signed_txn = storage.sign(LIBRA_ROOT_KEY, "remove-validator", txn)?;
        client.submit_transaction(signed_txn)
    }
}
