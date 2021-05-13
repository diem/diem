// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account_resource::SimplifiedAccountResource, validator_config::DecryptedValidatorConfig,
    validator_set::DecryptedValidatorInfo, TransactionContext,
};
use diem_config::config::Peer;
use diem_crypto::{ed25519::Ed25519PublicKey, x25519};
use diem_management::{error::Error, execute_command};
use diem_types::{account_address::AccountAddress, waypoint::Waypoint, PeerId};
use serde::Serialize;
use std::collections::HashMap;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(about = "Tool used for Operators")]
pub enum Command {
    #[structopt(about = "Displays the current account resource on the blockchain")]
    AccountResource(crate::account_resource::AccountResource),
    #[structopt(about = "Adds a validator to the ValidatorSet")]
    AddValidator(crate::governance::AddValidator),
    #[structopt(about = "Check an endpoint for a listening socket")]
    CheckEndpoint(crate::network_checker::CheckEndpoint),
    #[structopt(about = "Check all on-chain endpoints for a listening socket")]
    CheckValidatorSetEndpoints(crate::network_checker::CheckValidatorSetEndpoints),
    #[structopt(about = "Create a new validator account")]
    CreateValidator(crate::governance::CreateValidator),
    #[structopt(about = "Create a new validator operator account")]
    CreateValidatorOperator(crate::governance::CreateValidatorOperator),
    #[structopt(about = "Extract a trusted peer identity from an x25519 PrivateKey file")]
    ExtractPeerFromFile(crate::keys::ExtractPeerFromFile),
    #[structopt(about = "Extract a trusted peer identity from storage")]
    ExtractPeerFromStorage(crate::keys::ExtractPeerFromStorage),
    #[structopt(about = "Extract trusted peer identities from a list of Public Keys")]
    ExtractPeersFromKeys(crate::keys::ExtractPeersFromKeys),
    #[structopt(about = "Extract a private key from the validator storage")]
    ExtractPrivateKey(crate::keys::ExtractPrivateKey),
    #[structopt(about = "Extract a public key from the validator storage")]
    ExtractPublicKey(crate::keys::ExtractPublicKey),
    #[structopt(about = "Generate a PrivateKey to a file")]
    GenerateKey(crate::keys::GenerateKey),
    #[structopt(about = "Set the waypoint in the validator storage")]
    InsertWaypoint(diem_management::waypoint::InsertWaypoint),
    #[structopt(about = "Prints an account from the validator storage")]
    PrintAccount(crate::print::PrintAccount),
    #[structopt(about = "Prints an ed25519 public key from the validator storage")]
    PrintKey(crate::print::PrintKey),
    #[structopt(
        about = "Prints an x25519 public key from the validator storage, suitable for noise handshakes"
    )]
    PrintXKey(crate::print::PrintXKey),
    #[structopt(about = "Prints a waypoint from the validator storage")]
    PrintWaypoint(crate::print::PrintWaypoint),
    #[structopt(about = "Remove a validator from ValidatorSet")]
    RemoveValidator(crate::governance::RemoveValidator),
    #[structopt(about = "Rotates the consensus key for a validator")]
    RotateConsensusKey(crate::validator_config::RotateConsensusKey),
    #[structopt(about = "Rotates a full node network key")]
    RotateFullNodeNetworkKey(crate::validator_config::RotateFullNodeNetworkKey),
    #[structopt(about = "Rotates the operator key for the operator")]
    RotateOperatorKey(crate::account_resource::RotateOperatorKey),
    #[structopt(about = "Rotates a validator network key")]
    RotateValidatorNetworkKey(crate::validator_config::RotateValidatorNetworkKey),
    #[structopt(about = "Sets the validator config")]
    SetValidatorConfig(crate::validator_config::SetValidatorConfig),
    #[structopt(about = "Sets the validator operator")]
    SetValidatorOperator(crate::owner::SetValidatorOperator),
    #[structopt(about = "Validates a transaction")]
    ValidateTransaction(crate::validate_transaction::ValidateTransaction),
    #[structopt(about = "Displays the current validator config registered on the blockchain")]
    ValidatorConfig(crate::validator_config::ValidatorConfig),
    #[structopt(about = "Displays the current validator set infos registered on the blockchain")]
    ValidatorSet(crate::validator_set::ValidatorSet),
}

#[derive(Debug, PartialEq)]
pub enum CommandName {
    AccountResource,
    AddValidator,
    CheckEndpoint,
    CheckValidatorSetEndpoints,
    CreateValidator,
    CreateValidatorOperator,
    ExtractPeerFromFile,
    ExtractPeerFromStorage,
    ExtractPeersFromKeys,
    ExtractPrivateKey,
    ExtractPublicKey,
    GenerateKey,
    InsertWaypoint,
    PrintAccount,
    PrintKey,
    PrintXKey,
    PrintWaypoint,
    RemoveValidator,
    RotateConsensusKey,
    RotateOperatorKey,
    RotateFullNodeNetworkKey,
    RotateValidatorNetworkKey,
    SetValidatorConfig,
    SetValidatorOperator,
    ValidateTransaction,
    ValidatorConfig,
    ValidatorSet,
}

impl From<&Command> for CommandName {
    fn from(command: &Command) -> Self {
        match command {
            Command::AccountResource(_) => CommandName::AccountResource,
            Command::AddValidator(_) => CommandName::AddValidator,
            Command::CheckEndpoint(_) => CommandName::CheckEndpoint,
            Command::CheckValidatorSetEndpoints(_) => CommandName::CheckValidatorSetEndpoints,
            Command::CreateValidator(_) => CommandName::CreateValidator,
            Command::CreateValidatorOperator(_) => CommandName::CreateValidatorOperator,
            Command::ExtractPrivateKey(_) => CommandName::ExtractPrivateKey,
            Command::ExtractPublicKey(_) => CommandName::ExtractPublicKey,
            Command::ExtractPeerFromFile(_) => CommandName::ExtractPeerFromFile,
            Command::ExtractPeerFromStorage(_) => CommandName::ExtractPeerFromStorage,
            Command::ExtractPeersFromKeys(_) => CommandName::ExtractPeersFromKeys,
            Command::GenerateKey(_) => CommandName::GenerateKey,
            Command::InsertWaypoint(_) => CommandName::InsertWaypoint,
            Command::PrintAccount(_) => CommandName::PrintAccount,
            Command::PrintKey(_) => CommandName::PrintKey,
            Command::PrintXKey(_) => CommandName::PrintXKey,
            Command::PrintWaypoint(_) => CommandName::PrintWaypoint,
            Command::RemoveValidator(_) => CommandName::RemoveValidator,
            Command::RotateConsensusKey(_) => CommandName::RotateConsensusKey,
            Command::RotateOperatorKey(_) => CommandName::RotateOperatorKey,
            Command::RotateFullNodeNetworkKey(_) => CommandName::RotateFullNodeNetworkKey,
            Command::RotateValidatorNetworkKey(_) => CommandName::RotateValidatorNetworkKey,
            Command::SetValidatorConfig(_) => CommandName::SetValidatorConfig,
            Command::SetValidatorOperator(_) => CommandName::SetValidatorOperator,
            Command::ValidateTransaction(_) => CommandName::ValidateTransaction,
            Command::ValidatorConfig(_) => CommandName::ValidatorConfig,
            Command::ValidatorSet(_) => CommandName::ValidatorSet,
        }
    }
}

impl std::fmt::Display for CommandName {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let name = match self {
            CommandName::AccountResource => "account-resource",
            CommandName::AddValidator => "add-validator",
            CommandName::CheckEndpoint => "check-endpoint",
            CommandName::CheckValidatorSetEndpoints => "check-validator-set-endpoints",
            CommandName::CreateValidator => "create-validator",
            CommandName::CreateValidatorOperator => "create-validator-operator",
            CommandName::ExtractPrivateKey => "extract-private-key",
            CommandName::ExtractPublicKey => "extract-public-key",
            CommandName::ExtractPeerFromFile => "extract-peer-from-file",
            CommandName::ExtractPeerFromStorage => "extract-peer-from-storage",
            CommandName::ExtractPeersFromKeys => "extract-peers-from-keys",
            CommandName::GenerateKey => "generate-key",
            CommandName::InsertWaypoint => "insert-waypoint",
            CommandName::PrintAccount => "print-account",
            CommandName::PrintKey => "print-key",
            CommandName::PrintXKey => "print-x-key",
            CommandName::PrintWaypoint => "print-waypoint",
            CommandName::RemoveValidator => "remove-validator",
            CommandName::RotateConsensusKey => "rotate-consensus-key",
            CommandName::RotateOperatorKey => "rotate-operator-key",
            CommandName::RotateFullNodeNetworkKey => "rotate-full-node-network-key",
            CommandName::RotateValidatorNetworkKey => "rotate-validator-network-key",
            CommandName::SetValidatorConfig => "set-validator-config",
            CommandName::SetValidatorOperator => "set-validator-operator",
            CommandName::ValidateTransaction => "validate-transaction",
            CommandName::ValidatorConfig => "validator-config",
            CommandName::ValidatorSet => "validator-set",
        };
        write!(f, "{}", name)
    }
}

impl Command {
    pub fn execute(self) -> Result<String, Error> {
        match self {
            Command::AccountResource(cmd) => Self::pretty_print(cmd.execute()),
            Command::AddValidator(cmd) => Self::print_transaction_context(cmd.execute()),
            Command::CheckEndpoint(cmd) => Self::pretty_print(cmd.execute()),
            Command::CheckValidatorSetEndpoints(cmd) => Self::pretty_print(cmd.execute()),
            Command::CreateValidator(cmd) => {
                Self::print_transaction_context(cmd.execute().map(|(txn_ctx, _)| txn_ctx))
            }
            Command::CreateValidatorOperator(cmd) => {
                Self::print_transaction_context(cmd.execute().map(|(txn_ctx, _)| txn_ctx))
            }
            Command::InsertWaypoint(cmd) => Self::print_success(cmd.execute()),
            Command::ExtractPeerFromFile(cmd) => Self::pretty_print(cmd.execute()),
            Command::ExtractPeerFromStorage(cmd) => Self::pretty_print(cmd.execute()),
            Command::ExtractPeersFromKeys(cmd) => Self::pretty_print(cmd.execute()),
            Command::ExtractPrivateKey(cmd) => Self::print_success(cmd.execute()),
            Command::ExtractPublicKey(cmd) => Self::print_success(cmd.execute()),
            Command::GenerateKey(cmd) => Self::print_success(cmd.execute().map(|_| ())),
            Command::PrintAccount(cmd) => Self::pretty_print(cmd.execute()),
            Command::PrintKey(cmd) => Self::pretty_print(cmd.execute()),
            Command::PrintXKey(cmd) => Self::pretty_print(cmd.execute()),
            Command::PrintWaypoint(cmd) => Self::pretty_print(cmd.execute()),
            Command::RemoveValidator(cmd) => Self::print_transaction_context(cmd.execute()),
            Command::RotateConsensusKey(cmd) => {
                Self::print_transaction_context(cmd.execute().map(|(txn_ctx, _)| txn_ctx))
            }
            Command::RotateOperatorKey(cmd) => {
                Self::print_transaction_context(cmd.execute().map(|(txn_ctx, _)| txn_ctx))
            }
            Command::RotateFullNodeNetworkKey(cmd) => {
                Self::print_transaction_context(cmd.execute().map(|(txn_ctx, _)| txn_ctx))
            }
            Command::RotateValidatorNetworkKey(cmd) => {
                Self::print_transaction_context(cmd.execute().map(|(txn_ctx, _)| txn_ctx))
            }
            Command::SetValidatorConfig(cmd) => Self::print_transaction_context(cmd.execute()),
            Command::SetValidatorOperator(cmd) => Self::print_transaction_context(cmd.execute()),
            Command::ValidateTransaction(cmd) => Self::print_transaction_context(cmd.execute()),
            Command::ValidatorConfig(cmd) => Self::pretty_print(cmd.execute()),
            Command::ValidatorSet(cmd) => Self::pretty_print(cmd.execute()),
        }
    }

    /// Show the transaction context and validation result in a friendly way
    pub fn print_transaction_context(
        result: Result<TransactionContext, Error>,
    ) -> Result<String, Error> {
        match &result {
            Ok(txn_ctx) => match &txn_ctx.execution_result {
                Some(status) => Self::pretty_print(Ok(status.to_string())),
                None => Self::print_unvalidated_transaction_context(txn_ctx),
            },
            Err(_) => Self::pretty_print(result),
        }
    }

    /// Show a transaction context without an execution result
    fn print_unvalidated_transaction_context(
        transaction_context: &TransactionContext,
    ) -> Result<String, Error> {
        Self::pretty_print(Ok(UnvalidatedTransactionContext {
            address: transaction_context.address,
            sequence_number: transaction_context.sequence_number,
            execution_result: "Not yet validated.",
        }))
    }

    /// Show success or the error result
    fn print_success(result: Result<(), Error>) -> Result<String, Error> {
        Self::pretty_print(result.map(|()| "Success"))
    }

    /// For pretty printing outputs in JSON
    fn pretty_print<T: Serialize>(result: Result<T, Error>) -> Result<String, Error> {
        result.map(|val| serde_json::to_string_pretty(&ResultWrapper::Result(val)).unwrap())
    }

    pub fn account_resource(self) -> Result<SimplifiedAccountResource, Error> {
        execute_command!(self, Command::AccountResource, CommandName::AccountResource)
    }

    pub fn add_validator(self) -> Result<TransactionContext, Error> {
        execute_command!(self, Command::AddValidator, CommandName::AddValidator)
    }

    pub fn check_endpoint(self) -> Result<String, Error> {
        execute_command!(self, Command::CheckEndpoint, CommandName::CheckEndpoint)
    }

    pub fn check_validator_set_endpoints(self) -> Result<String, Error> {
        execute_command!(
            self,
            Command::CheckValidatorSetEndpoints,
            CommandName::CheckValidatorSetEndpoints
        )
    }

    pub fn create_validator(self) -> Result<(TransactionContext, AccountAddress), Error> {
        execute_command!(self, Command::CreateValidator, CommandName::CreateValidator)
    }

    pub fn create_validator_operator(self) -> Result<(TransactionContext, AccountAddress), Error> {
        execute_command!(
            self,
            Command::CreateValidatorOperator,
            CommandName::CreateValidatorOperator
        )
    }

    pub fn extract_private_key(self) -> Result<(), Error> {
        execute_command!(
            self,
            Command::ExtractPrivateKey,
            CommandName::ExtractPrivateKey
        )
    }

    pub fn extract_public_key(self) -> Result<(), Error> {
        execute_command!(
            self,
            Command::ExtractPublicKey,
            CommandName::ExtractPublicKey
        )
    }

    pub fn extract_peer_from_storage(self) -> Result<HashMap<PeerId, Peer>, Error> {
        execute_command!(
            self,
            Command::ExtractPeerFromStorage,
            CommandName::ExtractPeerFromStorage
        )
    }

    pub fn extract_peer_from_file(self) -> Result<HashMap<PeerId, Peer>, Error> {
        execute_command!(
            self,
            Command::ExtractPeerFromFile,
            CommandName::ExtractPeerFromFile
        )
    }

    pub fn extract_peers_from_keys(self) -> Result<HashMap<PeerId, Peer>, Error> {
        execute_command!(
            self,
            Command::ExtractPeersFromKeys,
            CommandName::ExtractPeersFromKeys
        )
    }

    pub fn generate_key(self) -> Result<(), Error> {
        execute_command!(self, Command::GenerateKey, CommandName::GenerateKey)
    }

    pub fn insert_waypoint(self) -> Result<(), Error> {
        execute_command!(self, Command::InsertWaypoint, CommandName::InsertWaypoint)
    }

    pub fn print_account(self) -> Result<AccountAddress, Error> {
        execute_command!(self, Command::PrintAccount, CommandName::PrintAccount)
    }

    pub fn print_key(self) -> Result<Ed25519PublicKey, Error> {
        execute_command!(self, Command::PrintKey, CommandName::PrintKey)
    }

    pub fn print_x_key(self) -> Result<x25519::PublicKey, Error> {
        execute_command!(self, Command::PrintXKey, CommandName::PrintXKey)
    }

    pub fn print_waypoint(self) -> Result<Waypoint, Error> {
        execute_command!(self, Command::PrintWaypoint, CommandName::PrintWaypoint)
    }

    pub fn remove_validator(self) -> Result<TransactionContext, Error> {
        execute_command!(self, Command::RemoveValidator, CommandName::RemoveValidator)
    }

    pub fn rotate_consensus_key(self) -> Result<(TransactionContext, Ed25519PublicKey), Error> {
        execute_command!(
            self,
            Command::RotateConsensusKey,
            CommandName::RotateConsensusKey
        )
    }

    pub fn rotate_operator_key(self) -> Result<(TransactionContext, Ed25519PublicKey), Error> {
        execute_command!(
            self,
            Command::RotateOperatorKey,
            CommandName::RotateOperatorKey
        )
    }

    pub fn rotate_fullnode_network_key(
        self,
    ) -> Result<(TransactionContext, x25519::PublicKey), Error> {
        execute_command!(
            self,
            Command::RotateFullNodeNetworkKey,
            CommandName::RotateFullNodeNetworkKey
        )
    }

    pub fn rotate_validator_network_key(
        self,
    ) -> Result<(TransactionContext, x25519::PublicKey), Error> {
        execute_command!(
            self,
            Command::RotateValidatorNetworkKey,
            CommandName::RotateValidatorNetworkKey
        )
    }

    pub fn set_validator_config(self) -> Result<TransactionContext, Error> {
        execute_command!(
            self,
            Command::SetValidatorConfig,
            CommandName::SetValidatorConfig
        )
    }

    pub fn set_validator_operator(self) -> Result<TransactionContext, Error> {
        execute_command!(
            self,
            Command::SetValidatorOperator,
            CommandName::SetValidatorOperator
        )
    }

    pub fn validate_transaction(self) -> Result<TransactionContext, Error> {
        execute_command!(
            self,
            Command::ValidateTransaction,
            CommandName::ValidateTransaction
        )
    }

    pub fn validator_config(self) -> Result<DecryptedValidatorConfig, Error> {
        execute_command!(self, Command::ValidatorConfig, CommandName::ValidatorConfig)
    }

    pub fn validator_set(self) -> Result<Vec<DecryptedValidatorInfo>, Error> {
        execute_command!(self, Command::ValidatorSet, CommandName::ValidatorSet)
    }
}

/// A result wrapper for displaying either a correct execution result or an error.
#[derive(Serialize)]
pub enum ResultWrapper<T> {
    Result(T),
    Error(String),
}

/// A struct wrapper for displaying unvalidated transaction contexts.
#[derive(Serialize)]
struct UnvalidatedTransactionContext<'a> {
    address: AccountAddress,
    sequence_number: u64,
    execution_result: &'a str,
}
