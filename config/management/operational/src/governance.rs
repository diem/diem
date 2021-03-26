// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{auto_validate::AutoValidate, json_rpc::JsonRpcClientWrapper, TransactionContext};
use diem_global_constants::DIEM_ROOT_KEY;
use diem_management::{
    config::{Config, ConfigPath},
    error::Error,
    secure_backend::ValidatorBackend,
    transaction::build_raw_transaction,
};
use diem_transaction_builder::stdlib as transaction_builder;
use diem_types::{
    account_address::AccountAddress,
    account_config::diem_root_address,
    chain_id::ChainId,
    transaction::{authenticator::AuthenticationKey, ScriptFunction, TransactionPayload},
};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct CreateAccount {
    #[structopt(flatten)]
    config: ConfigPath,
    #[structopt(long)]
    name: String,
    #[structopt(long)]
    path_to_key: PathBuf,
    #[structopt(long, required_unless = "config")]
    json_server: Option<String>,
    #[structopt(long, required_unless("config"))]
    chain_id: Option<ChainId>,
    #[structopt(flatten)]
    validator_backend: ValidatorBackend,
    #[structopt(flatten)]
    auto_validate: AutoValidate,
}

impl CreateAccount {
    fn execute(
        self,
        script_callback: fn(
            nonce: u64,
            account_address: AccountAddress,
            auth_key_prefix: Vec<u8>,
            name: Vec<u8>,
        ) -> TransactionPayload,
        action: &'static str,
    ) -> Result<(TransactionContext, AccountAddress), Error> {
        let config = self
            .config
            .load()?
            .override_chain_id(self.chain_id)
            .override_json_server(&self.json_server)
            .override_validator_backend(&self.validator_backend.validator_backend)?;

        let key = diem_management::read_key_from_file(&self.path_to_key)
            .map_err(|e| Error::UnableToReadFile(format!("{:?}", self.path_to_key), e))?;
        let client = JsonRpcClientWrapper::new(config.json_server.clone());

        let seq_num = client.sequence_number(diem_root_address())?;
        let auth_key = AuthenticationKey::ed25519(&key);
        let account_address = auth_key.derived_address();
        let script = script_callback(
            seq_num,
            account_address,
            auth_key.prefix().to_vec(),
            self.name.as_bytes().to_vec(),
        )
        .into_script_function();
        let mut transaction_context =
            build_and_submit_diem_root_transaction(&config, seq_num, script, action)?;

        // Perform auto validation if required
        transaction_context = self
            .auto_validate
            .execute(config.json_server, transaction_context)?;

        Ok((transaction_context, account_address))
    }
}

#[derive(Debug, StructOpt)]
pub struct CreateValidator {
    #[structopt(flatten)]
    input: CreateAccount,
}

impl CreateValidator {
    pub fn execute(self) -> Result<(TransactionContext, AccountAddress), Error> {
        self.input.execute(
            transaction_builder::encode_create_validator_account_script_function,
            "create-validator",
        )
    }
}

#[derive(Debug, StructOpt)]
pub struct CreateValidatorOperator {
    #[structopt(flatten)]
    input: CreateAccount,
}

impl CreateValidatorOperator {
    pub fn execute(self) -> Result<(TransactionContext, AccountAddress), Error> {
        self.input.execute(
            transaction_builder::encode_create_validator_operator_account_script_function,
            "create-validator-operator",
        )
    }
}

#[derive(Debug, StructOpt)]
struct RootValidatorOperation {
    #[structopt(long, help = "The validator address")]
    account_address: AccountAddress,
    /// JSON-RPC Endpoint (e.g. http://localhost:8080)
    #[structopt(long, required_unless = "config")]
    json_server: Option<String>,
    #[structopt(flatten)]
    validator_config: diem_management::validator_config::ValidatorConfig,
    #[structopt(flatten)]
    auto_validate: AutoValidate,
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

        let seq_num = client.sequence_number(diem_root_address())?;
        let script = transaction_builder::encode_add_validator_and_reconfigure_script_function(
            seq_num,
            name,
            self.input.account_address,
        )
        .into_script_function();
        let mut transaction_context =
            build_and_submit_diem_root_transaction(&config, seq_num, script, "add-validator")?;

        // Perform auto validation if required
        transaction_context = self
            .input
            .auto_validate
            .execute(config.json_server, transaction_context)?;

        Ok(transaction_context)
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

        let seq_num = client.sequence_number(diem_root_address())?;
        let script = transaction_builder::encode_remove_validator_and_reconfigure_script_function(
            seq_num,
            name,
            self.input.account_address,
        )
        .into_script_function();

        let mut transaction_context =
            build_and_submit_diem_root_transaction(&config, seq_num, script, "remove-validator")?;

        // Perform auto validation if required
        transaction_context = self
            .input
            .auto_validate
            .execute(config.json_server, transaction_context)?;

        Ok(transaction_context)
    }
}

fn build_and_submit_diem_root_transaction(
    config: &Config,
    seq_num: u64,
    script_function: ScriptFunction,
    action: &'static str,
) -> Result<TransactionContext, Error> {
    let txn = build_raw_transaction(
        config.chain_id,
        diem_root_address(),
        seq_num,
        script_function,
    );

    let mut storage = config.validator_backend();
    let signed_txn = storage.sign(DIEM_ROOT_KEY, action, txn)?;

    let client = JsonRpcClientWrapper::new(config.json_server.clone());
    client.submit_transaction(signed_txn)
}
