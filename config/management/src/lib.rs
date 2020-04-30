// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

mod error;

use crate::error::Error;
use libra_crypto::{ed25519::Ed25519PublicKey, hash::CryptoHash, x25519, ValidCryptoMaterial};
use libra_network_address::{NetworkAddress, RawNetworkAddress};
use libra_secure_storage::{Storage, VaultStorage};
use libra_secure_time::{RealTimeService, TimeService};
use libra_types::{
    account_address::AccountAddress,
    transaction::{RawTransaction, SignedTransaction, Transaction},
    waypoint::Waypoint,
};
use std::{
    convert::{TryFrom, TryInto},
    fmt::Write,
    str::FromStr,
    time::Duration,
};
use structopt::StructOpt;

pub mod constants {
    pub const ASSOCIATION_KEY: &str = "association";
    pub const CONSENSUS_KEY: &str = "consensus";
    pub const EPOCH: &str = "epoch";
    pub const FULLNODE_NETWORK_KEY: &str = "fullnode_network";
    pub const LAST_VOTED_ROUND: &str = "last_voted_round";
    pub const OWNER_KEY: &str = "owner";
    pub const OPERATOR_KEY: &str = "validator";
    pub const PREFERRED_ROUND: &str = "preferred_round";
    pub const VALIDATOR_NETWORK_KEY: &str = "validator_network";
    pub const WAYPOINT: &str = "waypoint";

    pub const GAS_UNIT_PRICE: u64 = 0;
    pub const MAX_GAS_AMOUNT: u64 = 1_000_000;
    pub const TXN_EXPIRATION_SECS: u64 = 3600;
}

#[derive(Debug, StructOpt)]
#[structopt(about = "Tool used to manage Libra Validators")]
pub enum Command {
    #[structopt(about = "Produces an LCS Ed25519PublicKey for the operator")]
    OperatorKey(SecureBackend),
    #[structopt(about = "Produces an LCS Ed25519PublicKey for the owner")]
    OwnerKey(SecureBackend),
    #[structopt(about = "Constructs and signs a ValidatorConfig")]
    ValidatorConfig(ValidatorConfig),
    #[structopt(about = "Verifies and prints the current configuration state")]
    Verify(SecureBackend),
}

pub enum CommandName {
    OperatorKey,
    OwnerKey,
    ValidatorConfig,
    Verify,
}

impl From<&Command> for CommandName {
    fn from(command: &Command) -> Self {
        match command {
            Command::OperatorKey(_) => CommandName::OperatorKey,
            Command::OwnerKey(_) => CommandName::OwnerKey,
            Command::ValidatorConfig(_) => CommandName::ValidatorConfig,
            Command::Verify(_) => CommandName::Verify,
        }
    }
}

impl std::fmt::Display for CommandName {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let name = match self {
            CommandName::OperatorKey => "operator-key",
            CommandName::OwnerKey => "owner-key",
            CommandName::ValidatorConfig => "validator-config",
            CommandName::Verify => "verify",
        };
        write!(f, "{}", name)
    }
}

impl Command {
    pub fn execute(self) -> String {
        match &self {
            Command::OperatorKey(_) => self.operator_key().unwrap().to_string(),
            Command::OwnerKey(_) => self.owner_key().unwrap().to_string(),
            Command::ValidatorConfig(_) => format!("{:?}", self.validator_config().unwrap()),
            Command::Verify(_) => self.verify().unwrap(),
        }
    }

    pub fn operator_key(self) -> Result<Ed25519PublicKey, Error> {
        if let Command::OperatorKey(secure_backend) = self {
            let storage = secure_storage(secure_backend);
            if !storage.available() {
                return Err(Error::LocalStorageUnavailable);
            }

            storage
                .get_public_key(constants::OPERATOR_KEY)
                .map_err(|e| Error::LocalStorageReadError(e.to_string()))
                .and_then(|v| Ok(v.public_key))
        } else {
            let expected = CommandName::OperatorKey.to_string();
            let actual = CommandName::from(&self).to_string();
            Err(Error::UnexpectedCommand(expected, actual))
        }
    }

    pub fn owner_key(self) -> Result<Ed25519PublicKey, Error> {
        if let Command::OwnerKey(secure_backend) = self {
            let storage = secure_storage(secure_backend);
            if !storage.available() {
                return Err(Error::LocalStorageUnavailable);
            }

            storage
                .get_public_key(constants::OWNER_KEY)
                .map_err(|e| Error::LocalStorageReadError(e.to_string()))
                .and_then(|v| Ok(v.public_key))
        } else {
            let expected = CommandName::OwnerKey.to_string();
            let actual = CommandName::from(&self).to_string();
            Err(Error::UnexpectedCommand(expected, actual))
        }
    }

    pub fn validator_config(self) -> Result<Transaction, Error> {
        if let Command::ValidatorConfig(config) = self {
            let mut storage = secure_storage(config.secure_backend);
            if !storage.available() {
                return Err(Error::LocalStorageUnavailable);
            }

            let consensus_key = storage
                .get_public_key(constants::CONSENSUS_KEY)
                .map_err(|e| Error::LocalStorageReadError(e.to_string()))?
                .public_key;
            let fullnode_network_key = storage
                .get_public_key(constants::FULLNODE_NETWORK_KEY)
                .map_err(|e| Error::LocalStorageReadError(e.to_string()))?
                .public_key;
            let validator_network_key = storage
                .get_public_key(constants::VALIDATOR_NETWORK_KEY)
                .map_err(|e| Error::LocalStorageReadError(e.to_string()))?
                .public_key;

            let fullnode_address = RawNetworkAddress::try_from(&config.fullnode_address)
                .expect("Unable to serialize fullnode network address from config");
            let validator_address = RawNetworkAddress::try_from(&config.validator_address)
                .expect("Unable to serialize validator network address from config");

            let fullnode_network_key = fullnode_network_key.to_bytes();
            let fullnode_network_key: x25519::PublicKey = fullnode_network_key
                .as_ref()
                .try_into()
                .expect("Unable to decode x25519 from fullnode_network_key");

            let validator_network_key = validator_network_key.to_bytes();
            let validator_network_key: x25519::PublicKey = validator_network_key
                .as_ref()
                .try_into()
                .expect("Unable to decode x25519 from validator_network_key");

            let owner_key = storage
                .get_public_key(constants::OWNER_KEY)
                .map_err(|e| Error::LocalStorageReadError(e.to_string()))?
                .public_key;

            // TODO(davidiw): The signing key, parameter 2, will be deleted soon, so this is a
            // temporary hack to reduce over-engineering.
            let script = transaction_builder::encode_register_validator_script(
                consensus_key.to_bytes().to_vec(),
                owner_key.to_bytes().to_vec(),
                validator_network_key.to_bytes(),
                validator_address.into(),
                fullnode_network_key.to_bytes(),
                fullnode_address.into(),
            );

            let sender = config.owner_address;
            // TODO(davidiw): In genesis this is irrelevant -- afterward we need to obtain the
            // current sequence number by querying the blockchain.
            let sequence_number = 0;
            let expiration_time = RealTimeService::new().now() + constants::TXN_EXPIRATION_SECS;
            let raw_transaction = RawTransaction::new_script(
                sender,
                sequence_number,
                script,
                constants::MAX_GAS_AMOUNT,
                constants::GAS_UNIT_PRICE,
                Duration::from_secs(expiration_time),
            );
            let signature = storage
                .sign_message(constants::OWNER_KEY, &raw_transaction.hash())
                .map_err(|e| Error::LocalStorageSigningError(e.to_string()))?;
            let signed_txn = SignedTransaction::new(raw_transaction, owner_key, signature);
            Ok(Transaction::UserTransaction(signed_txn))
        } else {
            let expected = CommandName::ValidatorConfig.to_string();
            let actual = CommandName::from(&self).to_string();
            Err(Error::UnexpectedCommand(expected, actual))
        }
    }

    pub fn verify(self) -> Result<String, Error> {
        if let Command::Verify(secure_backend) = self {
            let storage = secure_storage(secure_backend);
            if !storage.available() {
                return Err(Error::LocalStorageUnavailable);
            }

            let mut buffer = String::new();

            writeln!(buffer, "Data stored in SecureStorage:").unwrap();
            writeln!(buffer, "=================================================").unwrap();
            writeln!(buffer, "Keys").unwrap();
            writeln!(buffer, "=================================================").unwrap();

            Self::write_key(storage.as_ref(), &mut buffer, constants::CONSENSUS_KEY);
            Self::write_key(
                storage.as_ref(),
                &mut buffer,
                constants::FULLNODE_NETWORK_KEY,
            );
            Self::write_key(storage.as_ref(), &mut buffer, constants::OWNER_KEY);
            Self::write_key(storage.as_ref(), &mut buffer, constants::OPERATOR_KEY);
            Self::write_key(
                storage.as_ref(),
                &mut buffer,
                constants::VALIDATOR_NETWORK_KEY,
            );

            writeln!(buffer, "=================================================").unwrap();
            writeln!(buffer, "Data").unwrap();
            writeln!(buffer, "=================================================").unwrap();

            Self::write_u64(storage.as_ref(), &mut buffer, constants::EPOCH);
            Self::write_u64(storage.as_ref(), &mut buffer, constants::LAST_VOTED_ROUND);
            Self::write_u64(storage.as_ref(), &mut buffer, constants::PREFERRED_ROUND);
            Self::write_waypoint(storage.as_ref(), &mut buffer, constants::WAYPOINT);

            writeln!(buffer, "=================================================").unwrap();

            Ok(buffer)
        } else {
            panic!("Expected Command::Verify");
        }
    }

    fn write_key(storage: &dyn Storage, buffer: &mut String, key: &str) {
        let value = storage
            .get_public_key(key)
            .map(|c| c.public_key.to_string())
            .unwrap_or_else(|e| format!("{:?}", e));
        writeln!(buffer, "{} - {}", key, value).unwrap();
    }

    fn write_u64(storage: &dyn Storage, buffer: &mut String, key: &str) {
        let value = storage
            .get(key)
            .and_then(|c| c.value.u64())
            .map(|c| c.to_string())
            .unwrap_or_else(|e| format!("{:?}", e));
        writeln!(buffer, "{} - {}", key, value).unwrap();
    }

    fn write_waypoint(storage: &dyn Storage, buffer: &mut String, key: &str) {
        let value = storage
            .get(key)
            .and_then(|c| c.value.string())
            .map(|value| {
                if value.is_empty() {
                    "empty".into()
                } else {
                    Waypoint::from_str(&value)
                        .map(|c| c.to_string())
                        .unwrap_or_else(|_| "Invalid waypoint".into())
                }
            })
            .unwrap_or_else(|e| format!("{:?}", e));

        writeln!(buffer, "{} - {}", key, value).unwrap();
    }
}

#[derive(Debug, StructOpt)]
pub struct SecureBackend {
    #[structopt(long)]
    host: String,
    #[structopt(long)]
    token: String,
    #[structopt(long)]
    namespace: Option<String>,
}

// TODO(davidiw) add operator_address, since that will eventually be the identity producing this.
#[derive(Debug, StructOpt)]
pub struct ValidatorConfig {
    #[structopt(long)]
    owner_address: AccountAddress,
    #[structopt(long)]
    validator_address: NetworkAddress,
    #[structopt(long)]
    fullnode_address: NetworkAddress,
    #[structopt(flatten)]
    secure_backend: SecureBackend,
}

fn secure_storage(config: SecureBackend) -> Box<dyn Storage> {
    Box::new(VaultStorage::new(
        config.host,
        config.token,
        config.namespace,
    ))
}

/// These tests depends on running Vault, which can be done by using the provided docker run script
/// in `docker/vault/run.sh`.
/// Note: Some of these tests may fail if you run them too quickly one after another due to data
/// sychronization issues within Vault. It would seem the only way to fix it would be to restart
/// the Vault service between runs.
#[cfg(test)]
pub mod tests {
    use super::*;
    use libra_secure_storage::{Policy, Value};
    use libra_types::transaction::TransactionPayload;

    const VAULT_HOST: &str = "http://localhost:8200";
    const VAULT_ROOT_TOKEN: &str = "root_token";

    #[test]
    #[ignore]
    fn test_end_to_end() {
        let namespace = "end_to_end";
        let storage = initialize_storage(default_secure_backend(namespace.into()));
        let association_key = storage
            .get_public_key(constants::ASSOCIATION_KEY)
            .unwrap()
            .public_key;

        let validator_key = storage
            .get_public_key(constants::OWNER_KEY)
            .unwrap()
            .public_key;

        let validator_txn = validator_config(
            AccountAddress::random(),
            "/ip4/0.0.0.0/tcp/6180".parse().unwrap(),
            "/ip4/0.0.0.0/tcp/6180".parse().unwrap(),
            namespace,
        );

        let validator_txn = validator_txn.unwrap();
        let validator_txn = validator_txn.as_signed_user_txn().unwrap().payload();
        let validator_txn = if let TransactionPayload::Script(script) = validator_txn {
            script.clone()
        } else {
            panic!("Expected TransactionPayload::Script(_)");
        };

        vm_genesis::encode_genesis_transaction_with_validator(
            association_key,
            &[(validator_key, validator_txn)],
            None,
        );
    }

    #[test]
    #[ignore]
    fn test_validator_config() {
        let namespace = "validator_config";
        initialize_storage(default_secure_backend(namespace.into()));

        validator_config(
            AccountAddress::random(),
            "/ip4/0.0.0.0/tcp/6180".parse().unwrap(),
            "/ip4/0.0.0.0/tcp/6180".parse().unwrap(),
            namespace,
        )
        .unwrap();
    }

    #[test]
    #[ignore]
    fn test_verify() {
        let namespace = "verify";

        let mut storage = secure_storage(default_secure_backend(namespace.into()));
        storage.reset_and_clear().unwrap();

        let output = verify(namespace).unwrap().split("KeyNotSet").count();
        assert_eq!(output, 10); // 9 KeyNotSet results in 9 splits

        initialize_storage(default_secure_backend(namespace.into()));

        let output = verify(namespace).unwrap().split("KeyNotSet").count();
        assert_eq!(output, 1); // 0 KeyNotSet results in 1 split
    }

    #[test]
    #[ignore]
    fn test_owner_key() {
        let namespace = "owner_key";
        let mut storage = secure_storage(default_secure_backend(namespace.into()));
        storage.reset_and_clear().unwrap();
        owner_key(namespace).unwrap_err();

        let storage = initialize_storage(default_secure_backend(namespace.into()));
        let key = storage
            .get_public_key(constants::OWNER_KEY)
            .unwrap()
            .public_key;
        let okey = owner_key(namespace).unwrap();
        assert_eq!(key, okey);
    }

    #[test]
    #[ignore]
    fn test_operator_key_valid() {
        let namespace = "operator_key";
        let mut storage = secure_storage(default_secure_backend(namespace.into()));
        storage.reset_and_clear().unwrap();
        operator_key(namespace).unwrap_err();

        let storage = initialize_storage(default_secure_backend(namespace.into()));
        let key = storage
            .get_public_key(constants::OPERATOR_KEY)
            .unwrap()
            .public_key;
        let okey = operator_key(namespace).unwrap();
        assert_eq!(key, okey);
    }

    fn default_secure_backend(namespace: String) -> SecureBackend {
        SecureBackend {
            host: VAULT_HOST.into(),
            token: VAULT_ROOT_TOKEN.into(),
            namespace: Some(namespace),
        }
    }

    fn initialize_storage(config: SecureBackend) -> Box<dyn Storage> {
        let mut storage = secure_storage(config);
        let policy = Policy::public();
        storage.reset_and_clear().unwrap();

        storage
            .create_key(constants::ASSOCIATION_KEY, &policy)
            .unwrap();
        storage
            .create_key(constants::CONSENSUS_KEY, &policy)
            .unwrap();
        storage
            .create_key(constants::FULLNODE_NETWORK_KEY, &policy)
            .unwrap();
        storage.create_key(constants::OWNER_KEY, &policy).unwrap();
        storage
            .create_key(constants::OPERATOR_KEY, &policy)
            .unwrap();
        storage
            .create_key(constants::VALIDATOR_NETWORK_KEY, &policy)
            .unwrap();

        storage
            .create(constants::EPOCH, Value::U64(0), &policy)
            .unwrap();
        storage
            .create(constants::LAST_VOTED_ROUND, Value::U64(0), &policy)
            .unwrap();
        storage
            .create(constants::PREFERRED_ROUND, Value::U64(0), &policy)
            .unwrap();
        storage
            .create(constants::WAYPOINT, Value::String("".into()), &policy)
            .unwrap();

        storage
    }

    fn operator_key(namespace: &str) -> Result<Ed25519PublicKey, Error> {
        let args = format!(
            "
                management
                operator-key
                --host {secure_host}
                --token {secure_token}
                --namespace {secure_namespace}
            ",
            secure_host = VAULT_HOST,
            secure_token = VAULT_ROOT_TOKEN,
            secure_namespace = namespace,
        );

        let command = Command::from_iter(args.split_whitespace());
        command.operator_key()
    }

    fn owner_key(namespace: &str) -> Result<Ed25519PublicKey, Error> {
        let args = format!(
            "
                management
                owner-key
                --host {secure_host}
                --token {secure_token}
                --namespace {secure_namespace}
            ",
            secure_host = VAULT_HOST,
            secure_token = VAULT_ROOT_TOKEN,
            secure_namespace = namespace,
        );

        let command = Command::from_iter(args.split_whitespace());
        command.owner_key()
    }

    fn validator_config(
        owner_address: AccountAddress,
        validator_address: NetworkAddress,
        fullnode_address: NetworkAddress,
        namespace: &str,
    ) -> Result<Transaction, Error> {
        let args = format!(
            "
                management
                validator-config
                --owner-address {owner_address}
                --validator-address {validator_address}
                --fullnode-address {fullnode_address}
                --host {secure_host}
                --token {secure_token}
                --namespace {secure_namespace}
            ",
            owner_address = owner_address,
            validator_address = validator_address,
            fullnode_address = fullnode_address,
            secure_host = VAULT_HOST,
            secure_token = VAULT_ROOT_TOKEN,
            secure_namespace = namespace,
        );

        let command = Command::from_iter(args.split_whitespace());
        command.validator_config()
    }

    fn verify(namespace: &str) -> Result<String, Error> {
        let args = format!(
            "
                validator_config
                verify
                --host {secure_host}
                --token {secure_token}
                --namespace {secure_namespace}
            ",
            secure_host = VAULT_HOST,
            secure_token = VAULT_ROOT_TOKEN,
            secure_namespace = namespace,
        );

        let command = Command::from_iter(args.split_whitespace());
        command.verify()
    }
}
