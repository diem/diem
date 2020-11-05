// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account_resource::SimplifiedAccountResource,
    command::{Command, CommandName},
    validator_config::DecryptedValidatorConfig,
    validator_set::DecryptedValidatorInfo,
    TransactionContext,
};
use libra_config::config;
use libra_crypto::{ed25519::Ed25519PublicKey, x25519};
use libra_management::{error::Error, secure_backend::DISK};
use libra_network_address::NetworkAddress;
use libra_types::{account_address::AccountAddress, chain_id::ChainId, waypoint::Waypoint};
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

    pub fn account_resource(
        &self,
        account_address: AccountAddress,
    ) -> Result<SimplifiedAccountResource, Error> {
        let args = format!(
            "
                {command}
                --json-server {json_server}
                --account-address {account_address}
            ",
            command = command(TOOL_NAME, CommandName::AccountResource),
            json_server = self.host,
            account_address = account_address,
        );

        let command = Command::from_iter(args.split_whitespace());
        command.account_resource()
    }

    pub fn check_endpoint(&self, network_address: NetworkAddress) -> Result<String, Error> {
        let args = format!(
            "
                {command}
                --address {network_address}
            ",
            command = command(TOOL_NAME, CommandName::CheckEndpoint),
            network_address = network_address,
        );
        let command = Command::from_iter(args.split_whitespace());
        command.check_endpoint()
    }

    pub fn create_account(
        &self,
        name: &str,
        path_to_key: &str,
        backend: &config::SecureBackend,
        disable_validate: bool,
        command_name: CommandName,
        execute: fn(Command) -> Result<(TransactionContext, AccountAddress), Error>,
    ) -> Result<(TransactionContext, AccountAddress), Error> {
        let args = format!(
            "
                {command}
                --name {name}
                --path-to-key {path_to_key}
                --json-server {host}
                --chain-id {chain_id}
                --validator-backend {backend_args}
                {disable_validate}
            ",
            command = command(TOOL_NAME, command_name),
            name = name,
            path_to_key = path_to_key,
            host = self.host,
            chain_id = self.chain_id.id(),
            backend_args = backend_args(backend)?,
            disable_validate = optional_flag("disable-validate", disable_validate),
        );

        let command = Command::from_iter(args.split_whitespace());
        execute(command)
    }

    pub fn create_validator(
        &self,
        name: &str,
        path_to_key: &str,
        backend: &config::SecureBackend,
        disable_validate: bool,
    ) -> Result<(TransactionContext, AccountAddress), Error> {
        self.create_account(
            name,
            path_to_key,
            backend,
            disable_validate,
            CommandName::CreateValidator,
            |cmd| cmd.create_validator(),
        )
    }

    pub fn create_validator_operator(
        &self,
        name: &str,
        path_to_key: &str,
        backend: &config::SecureBackend,
        disable_validate: bool,
    ) -> Result<(TransactionContext, AccountAddress), Error> {
        self.create_account(
            name,
            path_to_key,
            backend,
            disable_validate,
            CommandName::CreateValidatorOperator,
            |cmd| cmd.create_validator_operator(),
        )
    }

    fn extract_key(
        &self,
        key_name: &str,
        key_file: &str,
        backend: &config::SecureBackend,
        command_name: CommandName,
        execute: fn(Command) -> Result<(), Error>,
    ) -> Result<(), Error> {
        let args = format!(
            "
                {command}
                --key-name {key_name}
                --key-file {key_file}
                --validator-backend {backend_args}
            ",
            command = command(TOOL_NAME, command_name),
            key_name = key_name,
            key_file = key_file,
            backend_args = backend_args(backend)?,
        );

        let command = Command::from_iter(args.split_whitespace());
        execute(command)
    }

    pub fn extract_public_key(
        &self,
        key_name: &str,
        key_file: &str,
        backend: &config::SecureBackend,
    ) -> Result<(), Error> {
        self.extract_key(
            key_name,
            key_file,
            backend,
            CommandName::ExtractPublicKey,
            |cmd| cmd.extract_public_key(),
        )
    }

    pub fn extract_private_key(
        &self,
        key_name: &str,
        key_file: &str,
        backend: &config::SecureBackend,
    ) -> Result<(), Error> {
        self.extract_key(
            key_name,
            key_file,
            backend,
            CommandName::ExtractPrivateKey,
            |cmd| cmd.extract_private_key(),
        )
    }

    pub fn insert_waypoint(
        &self,
        waypoint: Waypoint,
        backend: &config::SecureBackend,
        set_genesis: bool,
    ) -> Result<(), Error> {
        let args = format!(
            "
                {command}
                --waypoint {waypoint}
                --validator-backend {backend_args}
                {set_genesis}
            ",
            command = command(TOOL_NAME, CommandName::InsertWaypoint),
            waypoint = waypoint,
            backend_args = backend_args(backend)?,
            set_genesis = optional_flag("set-genesis", set_genesis),
        );
        let command = Command::from_iter(args.split_whitespace());
        command.insert_waypoint()
    }

    pub fn print_account(
        &self,
        account_name: &str,
        backend: &config::SecureBackend,
    ) -> Result<AccountAddress, Error> {
        let args = format!(
            "
                {command}
                --account-name {account_name}
                --validator-backend {backend_args}
            ",
            command = command(TOOL_NAME, CommandName::PrintAccount),
            account_name = account_name,
            backend_args = backend_args(backend)?,
        );
        let command = Command::from_iter(args.split_whitespace());
        command.print_account()
    }

    pub fn print_key(
        &self,
        key_name: &str,
        backend: &config::SecureBackend,
    ) -> Result<Ed25519PublicKey, Error> {
        let args = format!(
            "
                {command}
                --key-name {key_name}
                --validator-backend {backend_args}
            ",
            command = command(TOOL_NAME, CommandName::PrintKey),
            key_name = key_name,
            backend_args = backend_args(backend)?,
        );
        let command = Command::from_iter(args.split_whitespace());
        command.print_key()
    }

    pub fn print_waypoint(
        &self,
        waypoint_name: &str,
        backend: &config::SecureBackend,
    ) -> Result<Waypoint, Error> {
        let args = format!(
            "
                {command}
                --waypoint-name {waypoint_name}
                --validator-backend {backend_args}
            ",
            command = command(TOOL_NAME, CommandName::PrintWaypoint),
            waypoint_name = waypoint_name,
            backend_args = backend_args(backend)?,
        );
        let command = Command::from_iter(args.split_whitespace());
        command.print_waypoint()
    }

    pub fn set_validator_config(
        &self,
        validator_address: Option<NetworkAddress>,
        fullnode_address: Option<NetworkAddress>,
        backend: &config::SecureBackend,
        disable_validate: bool,
        disable_address_validation: bool,
    ) -> Result<TransactionContext, Error> {
        let args = format!(
            "
                {command}
                {fullnode_address}
                {validator_address}
                --chain-id {chain_id}
                --json-server {host}
                --validator-backend {backend_args}
                {disable_validate}
                {disable_address_validation}
            ",
            command = command(TOOL_NAME, CommandName::SetValidatorConfig),
            host = self.host,
            chain_id = self.chain_id.id(),
            fullnode_address = optional_arg("fullnode-address", fullnode_address),
            validator_address = optional_arg("validator-address", validator_address),
            backend_args = backend_args(backend)?,
            disable_validate = optional_flag("disable-validate", disable_validate),
            disable_address_validation =
                optional_flag("disable-address-validation", disable_address_validation),
        );

        let command = Command::from_iter(args.split_whitespace());
        command.set_validator_config()
    }

    fn rotate_key<T>(
        &self,
        backend: &config::SecureBackend,
        disable_validate: bool,
        name: CommandName,
        execute: fn(Command) -> Result<T, Error>,
    ) -> Result<T, Error> {
        let args = format!(
            "
                {command}
                --chain-id {chain_id}
                --json-server {host}
                --validator-backend {backend_args}
                {disable_validate}
            ",
            command = command(TOOL_NAME, name),
            host = self.host,
            chain_id = self.chain_id.id(),
            backend_args = backend_args(backend)?,
            disable_validate = optional_flag("disable-validate", disable_validate),
        );
        let command = Command::from_iter(args.split_whitespace());
        execute(command)
    }

    pub fn rotate_consensus_key(
        &self,
        backend: &config::SecureBackend,
        disable_validate: bool,
    ) -> Result<(TransactionContext, Ed25519PublicKey), Error> {
        self.rotate_key(
            backend,
            disable_validate,
            CommandName::RotateConsensusKey,
            |cmd| cmd.rotate_consensus_key(),
        )
    }

    pub fn rotate_operator_key(
        &self,
        backend: &config::SecureBackend,
        disable_validate: bool,
    ) -> Result<(TransactionContext, Ed25519PublicKey), Error> {
        self.rotate_key(
            backend,
            disable_validate,
            CommandName::RotateOperatorKey,
            |cmd| cmd.rotate_operator_key(),
        )
    }

    pub fn rotate_operator_key_with_custom_validation(
        &self,
        backend: &config::SecureBackend,
        disable_validate: bool,
        sleep_interval: Option<u64>,
        validate_timeout: Option<u64>,
    ) -> Result<(TransactionContext, Ed25519PublicKey), Error> {
        let args = format!(
            "
                {command}
                --chain-id {chain_id}
                --json-server {host}
                --validator-backend {backend_args}
                {disable_validate}
                {sleep_interval}
                {validate_timeout}
            ",
            command = command(TOOL_NAME, CommandName::RotateOperatorKey),
            host = self.host,
            chain_id = self.chain_id.id(),
            backend_args = backend_args(backend)?,
            disable_validate = optional_flag("disable-validate", disable_validate),
            sleep_interval = optional_arg("sleep-interval", sleep_interval),
            validate_timeout = optional_arg("validate-timeout", validate_timeout),
        );
        let command = Command::from_iter(args.split_whitespace());
        command.rotate_operator_key()
    }

    pub fn rotate_validator_network_key(
        &self,
        backend: &config::SecureBackend,
        disable_validate: bool,
    ) -> Result<(TransactionContext, x25519::PublicKey), Error> {
        self.rotate_key(
            backend,
            disable_validate,
            CommandName::RotateValidatorNetworkKey,
            |cmd| cmd.rotate_validator_network_key(),
        )
    }

    pub fn rotate_fullnode_network_key(
        &self,
        backend: &config::SecureBackend,
        disable_validate: bool,
    ) -> Result<(TransactionContext, x25519::PublicKey), Error> {
        self.rotate_key(
            backend,
            disable_validate,
            CommandName::RotateFullNodeNetworkKey,
            |cmd| cmd.rotate_fullnode_network_key(),
        )
    }

    pub fn validate_transaction(
        &self,
        account_address: AccountAddress,
        sequence_number: u64,
    ) -> Result<TransactionContext, Error> {
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

    pub fn set_validator_operator(
        &self,
        name: &str,
        account_address: AccountAddress,
        backend: &config::SecureBackend,
        disable_validate: bool,
    ) -> Result<TransactionContext, Error> {
        let args = format!(
            "
                {command}
                --json-server {json_server}
                --chain-id {chain_id}
                --name {name}
                --account-address {account_address}
                --validator-backend {backend_args}
                {disable_validate}
            ",
            command = command(TOOL_NAME, CommandName::SetValidatorOperator),
            json_server = self.host,
            name = name,
            chain_id = self.chain_id.id(),
            account_address = account_address,
            backend_args = backend_args(backend)?,
            disable_validate = optional_flag("disable-validate", disable_validate),
        );

        let command = Command::from_iter(args.split_whitespace());
        command.set_validator_operator()
    }

    pub fn validator_config(
        &self,
        account_address: AccountAddress,
        backend: &config::SecureBackend,
    ) -> Result<DecryptedValidatorConfig, Error> {
        let args = format!(
            "
                {command}
                --json-server {json_server}
                --account-address {account_address}
                --validator-backend {backend_args}
            ",
            command = command(TOOL_NAME, CommandName::ValidatorConfig),
            json_server = self.host,
            account_address = account_address,
            backend_args = backend_args(backend)?,
        );

        let command = Command::from_iter(args.split_whitespace());
        command.validator_config()
    }

    pub fn validator_set(
        &self,
        account_address: Option<AccountAddress>,
        backend: &config::SecureBackend,
    ) -> Result<Vec<DecryptedValidatorInfo>, Error> {
        let args = format!(
            "
                {command}
                {account_address}
                --json-server {json_server}
                --validator-backend {backend_args}
            ",
            command = command(TOOL_NAME, CommandName::ValidatorSet),
            json_server = self.host,
            account_address = optional_arg("account-address", account_address),
            backend_args = backend_args(backend)?,
        );

        let command = Command::from_iter(args.split_whitespace());
        command.validator_set()
    }

    fn validator_operation<T>(
        &self,
        account_address: AccountAddress,
        backend: &config::SecureBackend,
        disable_validate: bool,
        name: CommandName,
        execute: fn(Command) -> Result<T, Error>,
    ) -> Result<T, Error> {
        let args = format!(
            "
                {command}
                --json-server {host}
                --chain-id {chain_id}
                --account-address {account_address}
                --validator-backend {backend_args}
                {disable_validate}
            ",
            command = command(TOOL_NAME, name),
            host = self.host,
            chain_id = self.chain_id.id(),
            account_address = account_address,
            backend_args = backend_args(backend)?,
            disable_validate = optional_flag("disable-validate", disable_validate),
        );
        let command = Command::from_iter(args.split_whitespace());
        execute(command)
    }

    pub fn add_validator(
        &self,
        account_address: AccountAddress,
        backend: &config::SecureBackend,
        disable_validate: bool,
    ) -> Result<TransactionContext, Error> {
        self.validator_operation(
            account_address,
            backend,
            disable_validate,
            CommandName::AddValidator,
            |cmd| cmd.add_validator(),
        )
    }

    pub fn remove_validator(
        &self,
        account_address: AccountAddress,
        backend: &config::SecureBackend,
        disable_validate: bool,
    ) -> Result<TransactionContext, Error> {
        self.validator_operation(
            account_address,
            backend,
            disable_validate,
            CommandName::RemoveValidator,
            |cmd| cmd.remove_validator(),
        )
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

/// Allow flags to be optional
fn optional_flag(flag: &'static str, enable_flag: bool) -> String {
    if enable_flag {
        format!("--{flag}", flag = flag)
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
