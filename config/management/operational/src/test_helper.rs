// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    command::{Command, CommandName},
    TransactionContext,
};
use libra_config::config;
use libra_crypto::{ed25519::Ed25519PublicKey, x25519};
use libra_management::{error::Error, secure_backend::DISK};
use libra_network_address::NetworkAddress;
use libra_secure_json_rpc::VMStatusView;
use libra_types::{
    account_address::AccountAddress, chain_id::ChainId, validator_config::ValidatorConfig,
    validator_info::ValidatorInfo,
};
use structopt::StructOpt;

const TOOL_NAME: &str = "libra-operational-tool";

/// A helper to test the operational tool in tests
pub struct OperationalTool {
    host: String,
    chain_id: ChainId,
}

impl OperationalTool {
    pub fn new(host: String, chain_id: ChainId) -> OperationalTool {
        OperationalTool { host, chain_id }
    }

    pub fn set_validator_config(
        &self,
        validator_address: Option<NetworkAddress>,
        fullnode_address: Option<NetworkAddress>,
    ) -> Result<TransactionContext, Error> {
        let args = format!(
            "
                {command}
                {fullnode_address}
                {validator_address}
                --chain-id {chain_id}
                --json-server {host}
            ",
            command = command(TOOL_NAME, CommandName::SetValidatorConfig),
            host = self.host,
            chain_id = self.chain_id.id(),
            fullnode_address = optional_arg("fullnode-address", fullnode_address),
            validator_address = optional_arg("validator-address", validator_address),
        );

        let command = Command::from_iter(args.split_whitespace());
        command.set_validator_config()
    }

    fn rotate_key<T>(
        &self,
        backend: &config::SecureBackend,
        name: CommandName,
        execute: fn(Command) -> Result<T, Error>,
    ) -> Result<T, Error> {
        let args = format!(
            "
                {command}
                --chain-id {chain_id}
                --json-server {host}
                --validator-backend {backend_args}
            ",
            command = command(TOOL_NAME, name),
            host = self.host,
            chain_id = self.chain_id.id(),
            backend_args = backend_args(backend)?,
        );
        let command = Command::from_iter(args.split_whitespace());
        execute(command)
    }

    pub fn rotate_consensus_key(
        &self,
        backend: &config::SecureBackend,
    ) -> Result<(TransactionContext, Ed25519PublicKey), Error> {
        self.rotate_key(backend, CommandName::RotateConsensusKey, |cmd| {
            cmd.rotate_consensus_key()
        })
    }

    pub fn rotate_operator_key(
        &self,
        backend: &config::SecureBackend,
    ) -> Result<(TransactionContext, Ed25519PublicKey), Error> {
        self.rotate_key(backend, CommandName::RotateOperatorKey, |cmd| {
            cmd.rotate_operator_key()
        })
    }

    pub fn rotate_validator_network_key(
        &self,
        backend: &config::SecureBackend,
    ) -> Result<(TransactionContext, x25519::PublicKey), Error> {
        self.rotate_key(backend, CommandName::RotateValidatorNetworkKey, |cmd| {
            cmd.rotate_validator_network_key()
        })
    }

    pub fn rotate_fullnode_network_key(
        &self,
        backend: &config::SecureBackend,
    ) -> Result<(TransactionContext, x25519::PublicKey), Error> {
        self.rotate_key(backend, CommandName::RotateFullNodeNetworkKey, |cmd| {
            cmd.rotate_fullnode_network_key()
        })
    }

    pub fn validate_transaction(
        &self,
        account_address: AccountAddress,
        sequence_number: u64,
    ) -> Result<Option<VMStatusView>, Error> {
        let args = format!(
            "
                {command}
                --json-server {host}
                --account-address {account_address}
                --sequence-number {sequence_number}
        ",
            command = command(TOOL_NAME, CommandName::ValidateTransaction),
            host = self.host,
            account_address = account_address,
            sequence_number = sequence_number,
        );

        let command = Command::from_iter(args.split_whitespace());
        command.validate_transaction()
    }

    pub fn validator_config(
        &self,
        account_address: AccountAddress,
    ) -> Result<ValidatorConfig, Error> {
        let args = format!(
            "
                {command}
                --json-server {json_server}
                --account-address {account_address}
        ",
            command = command(TOOL_NAME, CommandName::ValidatorConfig),
            json_server = self.host,
            account_address = account_address,
        );

        let command = Command::from_iter(args.split_whitespace());
        command.validator_config()
    }

    pub fn validator_set(
        &self,
        account_address: AccountAddress,
    ) -> Result<Vec<ValidatorInfo>, Error> {
        let args = format!(
            "
                {command}
                --json-server {json_server}
                --account-address {account_address}
        ",
            command = command(TOOL_NAME, CommandName::ValidatorSet),
            json_server = self.host,
            account_address = account_address,
        );

        let command = Command::from_iter(args.split_whitespace());
        command.validator_set()
    }

    pub fn add_validator(
        &self,
        account_address: AccountAddress,
        backend: &config::SecureBackend,
    ) -> Result<TransactionContext, Error> {
        let args = format!(
            "
            {command}
            --json-server {host}
            --chain-id {chain_id}
            --account-address {account_address}
            --validator-backend {backend_args}
            ",
            command = command(TOOL_NAME, CommandName::AddValidator),
            host = self.host,
            chain_id = self.chain_id.id(),
            account_address = account_address,
            backend_args = backend_args(backend)?,
        );
        let command = Command::from_iter(args.split_whitespace());
        command.add_validator()
    }

    pub fn remove_validator(
        &self,
        account_address: AccountAddress,
        backend: &config::SecureBackend,
    ) -> Result<TransactionContext, Error> {
        let args = format!(
            "
            {command}
            --json-server {host}
            --chain-id {chain_id}
            --account-address {account_address}
            --validator-backend {backend_args}
            ",
            command = command(TOOL_NAME, CommandName::RemoveValidator),
            host = self.host,
            chain_id = self.chain_id.id(),
            account_address = account_address,
            backend_args = backend_args(backend)?,
        );
        let command = Command::from_iter(args.split_whitespace());
        command.remove_validator()
    }
}

fn command(tool_name: &'static str, command: CommandName) -> String {
    format!("{tool} {command}", tool = tool_name, command = command)
}

/// Allow arguments to be optional
fn optional_arg<T: std::fmt::Display>(name: &'static str, maybe_value: Option<T>) -> String {
    if let Some(value) = maybe_value {
        format!("--{name} {value}", name = name, value = value)
    } else {
        String::new()
    }
}

/// Extract on disk storage args
/// TODO: Support other types of storage
fn backend_args(backend: &config::SecureBackend) -> Result<String, Error> {
    match backend {
        config::SecureBackend::OnDiskStorage(config) => Ok(format!(
            "backend={backend};\
            path={path};\
            namespace={namespace}",
            backend = DISK,
            namespace = config.namespace.clone().unwrap(),
            path = config.path.to_str().unwrap(),
        )),
        _ => Err(Error::UnexpectedError("Storage isn't on disk".to_string())),
    }
}
