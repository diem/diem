// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0
use libra_crypto::{ed25519::Ed25519PublicKey, x25519};
use libra_management::{error::Error, TransactionContext};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(about = "Tool used for Operators")]
pub enum Command {
    #[structopt(about = "Sets the validator config")]
    SetValidatorConfig(crate::validator_config::SetValidatorConfig),
    #[structopt(about = "Rotates the consensus key for a validator")]
    RotateConsensusKey(crate::validator_config::RotateConsensusKey),
    #[structopt(about = "Rotates a full node network key")]
    RotateFullNodeNetworkKey(crate::validator_config::RotateFullNodeNetworkKey),
    #[structopt(about = "Rotates a validator network key")]
    RotateValidatorNetworkKey(crate::validator_config::RotateValidatorNetworkKey),
    #[structopt(about = "Validates a transaction")]
    ValidateTransaction(crate::validate_transaction::ValidateTransaction),
}

#[derive(Debug, PartialEq)]
pub enum CommandName {
    SetValidatorConfig,
    RotateConsensusKey,
    RotateFullNodeNetworkKey,
    RotateValidatorNetworkKey,
    ValidateTransaction,
}

impl From<&Command> for CommandName {
    fn from(command: &Command) -> Self {
        match command {
            Command::SetValidatorConfig(_) => CommandName::SetValidatorConfig,
            Command::RotateConsensusKey(_) => CommandName::RotateConsensusKey,
            Command::RotateFullNodeNetworkKey(_) => CommandName::RotateFullNodeNetworkKey,
            Command::RotateValidatorNetworkKey(_) => CommandName::RotateValidatorNetworkKey,
            Command::ValidateTransaction(_) => CommandName::ValidateTransaction,
        }
    }
}

impl std::fmt::Display for CommandName {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let name = match self {
            CommandName::SetValidatorConfig => "set-validator-config",
            CommandName::RotateConsensusKey => "rotate-consensus-key",
            CommandName::RotateFullNodeNetworkKey => "rotate-fullnode-network-key",
            CommandName::RotateValidatorNetworkKey => "rotate-validator-network-key",
            CommandName::ValidateTransaction => "validate-transaction",
        };
        write!(f, "{}", name)
    }
}

impl Command {
    pub fn execute(self) -> String {
        match self {
            Command::SetValidatorConfig(cmd) => format!("{:?}", cmd.execute().unwrap()),
            Command::RotateConsensusKey(cmd) => format!("{:?}", cmd.execute().unwrap()),
            Command::RotateFullNodeNetworkKey(cmd) => format!("{:?}", cmd.execute().unwrap()),
            Command::RotateValidatorNetworkKey(cmd) => format!("{:?}", cmd.execute().unwrap()),
            Command::ValidateTransaction(cmd) => cmd.execute().unwrap().to_string(),
        }
    }

    pub fn set_validator_config(self) -> Result<TransactionContext, Error> {
        match self {
            Command::SetValidatorConfig(cmd) => cmd.execute(),
            _ => Err(self.unexpected_command(CommandName::SetValidatorConfig)),
        }
    }

    pub fn rotate_consensus_key(self) -> Result<(TransactionContext, Ed25519PublicKey), Error> {
        match self {
            Command::RotateConsensusKey(cmd) => cmd.execute(),
            _ => Err(self.unexpected_command(CommandName::RotateConsensusKey)),
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

    pub fn validate_transaction(self) -> Result<bool, Error> {
        match self {
            Command::ValidateTransaction(cmd) => cmd.execute(),
            _ => Err(self.unexpected_command(CommandName::ValidateTransaction)),
        }
    }

    fn unexpected_command(self, expected: CommandName) -> Error {
        Error::UnexpectedCommand(expected.to_string(), CommandName::from(&self).to_string())
    }
}
