// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::TransactionContext;
use libra_crypto::{ed25519::Ed25519PublicKey, x25519};
use libra_management::error::Error;
use libra_secure_json_rpc::VMStatusView;
use libra_types::{validator_config::ValidatorConfig, validator_info::ValidatorInfo};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(about = "Tool used for Operators")]
pub enum Command {
    #[structopt(about = "Remove a validator from ValidatorSet")]
    AddValidator(crate::governance::AddValidator),
    #[structopt(about = "Extract a private key from the validator storage")]
    ExtractPrivateKey(crate::keys::ExtractPrivateKey),
    #[structopt(about = "Extract a public key from the validator storage")]
    ExtractPublicKey(crate::keys::ExtractPublicKey),
    #[structopt(about = "Set the waypoint in the validator storage")]
    InsertWaypoint(crate::waypoint::InsertWaypoint),
    #[structopt(about = "Remove a validator from ValidatorSet")]
    RemoveValidator(crate::governance::RemoveValidator),
    #[structopt(about = "Rotates the consensus key for a validator")]
    RotateConsensusKey(crate::validator_config::RotateConsensusKey),
    #[structopt(about = "Rotates a full node network key")]
    RotateFullNodeNetworkKey(crate::validator_config::RotateFullNodeNetworkKey),
    #[structopt(about = "Rotates the operator key for the operator")]
    RotateOperatorKey(crate::operator_key::RotateOperatorKey),
    #[structopt(about = "Rotates a validator network key")]
    RotateValidatorNetworkKey(crate::validator_config::RotateValidatorNetworkKey),
    #[structopt(about = "Sets the validator config")]
    SetValidatorConfig(crate::validator_config::SetValidatorConfig),
    #[structopt(about = "Validates a transaction")]
    ValidateTransaction(crate::validate_transaction::ValidateTransaction),
    #[structopt(about = "Displays the current validator config registered on the blockchain")]
    ValidatorConfig(crate::validator_config::ValidatorConfig),
    #[structopt(about = "Displays the current validator set infos registered on the blockchain")]
    ValidatorSet(crate::validator_set::ValidatorSet),
}

#[derive(Debug, PartialEq)]
pub enum CommandName {
    AddValidator,
    ExtractPrivateKey,
    ExtractPublicKey,
    InsertWaypoint,
    RemoveValidator,
    RotateConsensusKey,
    RotateOperatorKey,
    RotateFullNodeNetworkKey,
    RotateValidatorNetworkKey,
    SetValidatorConfig,
    ValidateTransaction,
    ValidatorConfig,
    ValidatorSet,
}

impl From<&Command> for CommandName {
    fn from(command: &Command) -> Self {
        match command {
            Command::AddValidator(_) => CommandName::AddValidator,
            Command::ExtractPrivateKey(_) => CommandName::ExtractPrivateKey,
            Command::ExtractPublicKey(_) => CommandName::ExtractPublicKey,
            Command::InsertWaypoint(_) => CommandName::InsertWaypoint,
            Command::RemoveValidator(_) => CommandName::RemoveValidator,
            Command::RotateConsensusKey(_) => CommandName::RotateConsensusKey,
            Command::RotateOperatorKey(_) => CommandName::RotateOperatorKey,
            Command::RotateFullNodeNetworkKey(_) => CommandName::RotateFullNodeNetworkKey,
            Command::RotateValidatorNetworkKey(_) => CommandName::RotateValidatorNetworkKey,
            Command::SetValidatorConfig(_) => CommandName::SetValidatorConfig,
            Command::ValidateTransaction(_) => CommandName::ValidateTransaction,
            Command::ValidatorConfig(_) => CommandName::ValidatorConfig,
            Command::ValidatorSet(_) => CommandName::ValidatorSet,
        }
    }
}

impl std::fmt::Display for CommandName {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let name = match self {
            CommandName::AddValidator => "add-validator",
            CommandName::ExtractPrivateKey => "extract-private-key",
            CommandName::ExtractPublicKey => "extract-public-key",
            CommandName::InsertWaypoint => "insert-waypoint",
            CommandName::RemoveValidator => "remove-validator",
            CommandName::RotateConsensusKey => "rotate-consensus-key",
            CommandName::RotateOperatorKey => "rotate-operator-key",
            CommandName::RotateFullNodeNetworkKey => "rotate-fullnode-network-key",
            CommandName::RotateValidatorNetworkKey => "rotate-validator-network-key",
            CommandName::SetValidatorConfig => "set-validator-config",
            CommandName::ValidateTransaction => "validate-transaction",
            CommandName::ValidatorConfig => "validator-config",
            CommandName::ValidatorSet => "validator-set",
        };
        write!(f, "{}", name)
    }
}

impl Command {
    pub fn execute(self) -> String {
        match self {
            Command::AddValidator(cmd) => format!("{:?}", cmd.execute().unwrap()),
            Command::InsertWaypoint(cmd) => cmd.execute().map(|()| "success".into()).unwrap(),
            Command::ExtractPrivateKey(cmd) => cmd.execute().map(|()| "success".into()).unwrap(),
            Command::ExtractPublicKey(cmd) => cmd.execute().map(|()| "success".into()).unwrap(),
            Command::RemoveValidator(cmd) => format!("{:?}", cmd.execute().unwrap()),
            Command::RotateConsensusKey(cmd) => format!("{:?}", cmd.execute().unwrap()),
            Command::RotateOperatorKey(cmd) => format!("{:?}", cmd.execute().unwrap()),
            Command::RotateFullNodeNetworkKey(cmd) => format!("{:?}", cmd.execute().unwrap()),
            Command::RotateValidatorNetworkKey(cmd) => format!("{:?}", cmd.execute().unwrap()),
            Command::SetValidatorConfig(cmd) => format!("{:?}", cmd.execute().unwrap()),
            Command::ValidateTransaction(cmd) => format!("{:?}", cmd.execute().unwrap()),
            Command::ValidatorConfig(cmd) => format!("{:?}", cmd.execute().unwrap()),
            Command::ValidatorSet(cmd) => format!("{:?}", cmd.execute().unwrap()),
        }
    }

    pub fn add_validator(self) -> Result<TransactionContext, Error> {
        match self {
            Command::AddValidator(cmd) => cmd.execute(),
            _ => Err(self.unexpected_command(CommandName::AddValidator)),
        }
    }

    pub fn extract_private_key(self) -> Result<(), Error> {
        match self {
            Command::ExtractPrivateKey(cmd) => cmd.execute(),
            _ => Err(self.unexpected_command(CommandName::ExtractPrivateKey)),
        }
    }

    pub fn extract_public_key(self) -> Result<(), Error> {
        match self {
            Command::ExtractPublicKey(cmd) => cmd.execute(),
            _ => Err(self.unexpected_command(CommandName::ExtractPublicKey)),
        }
    }

    pub fn insert_waypoint(self) -> Result<(), Error> {
        match self {
            Command::InsertWaypoint(cmd) => cmd.execute(),
            _ => Err(self.unexpected_command(CommandName::InsertWaypoint)),
        }
    }

    pub fn remove_validator(self) -> Result<TransactionContext, Error> {
        match self {
            Command::RemoveValidator(cmd) => cmd.execute(),
            _ => Err(self.unexpected_command(CommandName::RemoveValidator)),
        }
    }

    pub fn rotate_consensus_key(self) -> Result<(TransactionContext, Ed25519PublicKey), Error> {
        match self {
            Command::RotateConsensusKey(cmd) => cmd.execute(),
            _ => Err(self.unexpected_command(CommandName::RotateConsensusKey)),
        }
    }

    pub fn rotate_operator_key(self) -> Result<(TransactionContext, Ed25519PublicKey), Error> {
        match self {
            Command::RotateOperatorKey(cmd) => cmd.execute(),
            _ => Err(self.unexpected_command(CommandName::RotateOperatorKey)),
        }
    }

    pub fn rotate_fullnode_network_key(
        self,
    ) -> Result<(TransactionContext, x25519::PublicKey), Error> {
        match self {
            Command::RotateFullNodeNetworkKey(cmd) => cmd.execute(),
            _ => Err(self.unexpected_command(CommandName::RotateFullNodeNetworkKey)),
        }
    }

    pub fn rotate_validator_network_key(
        self,
    ) -> Result<(TransactionContext, x25519::PublicKey), Error> {
        match self {
            Command::RotateValidatorNetworkKey(cmd) => cmd.execute(),
            _ => Err(self.unexpected_command(CommandName::RotateValidatorNetworkKey)),
        }
    }

    pub fn set_validator_config(self) -> Result<TransactionContext, Error> {
        match self {
            Command::SetValidatorConfig(cmd) => cmd.execute(),
            _ => Err(self.unexpected_command(CommandName::SetValidatorConfig)),
        }
    }

    pub fn validate_transaction(self) -> Result<Option<VMStatusView>, Error> {
        match self {
            Command::ValidateTransaction(cmd) => cmd.execute(),
            _ => Err(self.unexpected_command(CommandName::ValidateTransaction)),
        }
    }

    pub fn validator_config(self) -> Result<ValidatorConfig, Error> {
        match self {
            Command::ValidatorConfig(cmd) => cmd.execute(),
            _ => Err(self.unexpected_command(CommandName::ValidatorConfig)),
        }
    }

    pub fn validator_set(self) -> Result<Vec<ValidatorInfo>, Error> {
        match self {
            Command::ValidatorSet(cmd) => cmd.execute(),
            _ => Err(self.unexpected_command(CommandName::ValidatorSet)),
        }
    }

    fn unexpected_command(self, expected: CommandName) -> Error {
        Error::UnexpectedCommand(expected.to_string(), CommandName::from(&self).to_string())
    }
}
