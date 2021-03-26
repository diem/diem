// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{auto_validate::AutoValidate, json_rpc::JsonRpcClientWrapper, TransactionContext};
use diem_management::{
    config::ConfigPath, error::Error, secure_backend::ValidatorBackend,
    transaction::build_raw_transaction,
};
use diem_transaction_builder::stdlib as transaction_builder;
use diem_types::{account_address::AccountAddress, chain_id::ChainId};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct SetValidatorOperator {
    #[structopt(flatten)]
    config: ConfigPath,
    #[structopt(long)]
    name: String,
    #[structopt(long)]
    account_address: AccountAddress,
    #[structopt(long, required_unless = "config")]
    json_server: Option<String>,
    #[structopt(long, required_unless("config"))]
    chain_id: Option<ChainId>,
    #[structopt(flatten)]
    validator_backend: ValidatorBackend,
    #[structopt(flatten)]
    auto_validate: AutoValidate,
}

impl SetValidatorOperator {
    pub fn execute(self) -> Result<TransactionContext, Error> {
        let config = self
            .config
            .load()?
            .override_chain_id(self.chain_id)
            .override_json_server(&self.json_server)
            .override_validator_backend(&self.validator_backend.validator_backend)?;
        let mut storage = config.validator_backend();
        let owner_address = storage.account_address(diem_global_constants::OWNER_ACCOUNT)?;

        let client = JsonRpcClientWrapper::new(config.json_server.clone());
        let txn = build_raw_transaction(
            config.chain_id,
            owner_address,
            client.sequence_number(owner_address)?,
            transaction_builder::encode_set_validator_operator_script_function(
                self.name.as_bytes().to_vec(),
                self.account_address,
            )
            .into_script_function(),
        );

        let signed_txn = storage.sign(diem_global_constants::OWNER_KEY, "set-operator", txn)?;
        let mut transaction_context = client.submit_transaction(signed_txn)?;

        // Perform auto validation if required
        transaction_context = self
            .auto_validate
            .execute(config.json_server, transaction_context)?;

        Ok(transaction_context)
    }
}
