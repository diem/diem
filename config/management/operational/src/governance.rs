// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{json_rpc::JsonRpcClientWrapper, TransactionContext};
use libra_global_constants::LIBRA_ROOT_KEY;
use libra_management::{
    config::{Config, ConfigPath},
    error::Error,
    transaction::build_raw_transaction,
};
use libra_types::{
    account_address::AccountAddress,
    account_config::libra_root_address,
    chain_id::ChainId,
    transaction::{authenticator::AuthenticationKey, Script},
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
}

impl CreateAccount {
    fn execute(
        self,
        script_callback: fn(
            nonce: u64,
            account_address: AccountAddress,
            auth_key_prefix: Vec<u8>,
            name: Vec<u8>,
        ) -> Script,
        action: &'static str,
    ) -> Result<(TransactionContext, AccountAddress), Error> {
        let config = self
            .config
            .load()?
            .override_chain_id(self.chain_id)
            .override_json_server(&self.json_server);
        let key = libra_management::read_key_from_file(&self.path_to_key)
            .map_err(|e| Error::UnableToReadFile(format!("{:?}", self.path_to_key), e))?;
        let client = JsonRpcClientWrapper::new(config.json_server.clone());

        let seq_num = client.sequence_number(libra_root_address())?;
        let auth_key = AuthenticationKey::ed25519(&key);
        let account_address = auth_key.derived_address();
        let script = script_callback(
            seq_num,
            account_address,
            auth_key.prefix().to_vec(),
            self.name.as_bytes().to_vec(),
        );
        build_and_submit_libra_root_transaction(&config, seq_num, script, action)
            .map(|a| (a, account_address))
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
            transaction_builder::encode_create_validator_account_script,
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
            transaction_builder::encode_create_validator_operator_account_script,
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

        let seq_num = client.sequence_number(libra_root_address())?;
        let script = transaction_builder::encode_add_validator_and_reconfigure_script(
            seq_num,
            name,
            self.input.account_address,
        );
        build_and_submit_libra_root_transaction(&config, seq_num, script, "add-validator")
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

        let seq_num = client.sequence_number(libra_root_address())?;
        let script = transaction_builder::encode_remove_validator_and_reconfigure_script(
            seq_num,
            name,
            self.input.account_address,
        );

        build_and_submit_libra_root_transaction(&config, seq_num, script, "remove-validator")
    }
}

fn build_and_submit_libra_root_transaction(
    config: &Config,
    seq_num: u64,
    script: Script,
    action: &'static str,
) -> Result<TransactionContext, Error> {
    let txn = build_raw_transaction(config.chain_id, libra_root_address(), seq_num, script);

    let mut storage = config.validator_backend();
    let signed_txn = storage.sign(LIBRA_ROOT_KEY, action, txn)?;

    let client = JsonRpcClientWrapper::new(config.json_server.clone());
    client.submit_transaction(signed_txn)
}
