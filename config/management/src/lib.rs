// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use libra_secure_storage::{Storage, VaultStorage};
use libra_types::waypoint::Waypoint;
use std::{fmt::Write, str::FromStr};
use structopt::StructOpt;

pub mod constants {
    pub const CONSENSUS_KEY: &str = "consensus";
    pub const EPOCH: &str = "epoch";
    pub const FULLNODE_NETWORK_KEY: &str = "fullnode_network";
    pub const LAST_VOTED_ROUND: &str = "last_voted_round";
    pub const OWNER_KEY: &str = "owner";
    pub const OPERATOR_KEY: &str = "validator";
    pub const PREFERRED_ROUND: &str = "preferred_round";
    pub const VALIDATOR_NETWORK_KEY: &str = "validator_network";
    pub const WAYPOINT: &str = "waypoint";
}

#[derive(Debug, StructOpt)]
#[structopt(about = "Tool used to manage Libra Validators")]
pub enum Command {
    #[structopt(about = "Dummy command until more implementation exists")]
    NotImplemented,
    #[structopt(about = "Verifies and prints the current configuration state")]
    Verify(SecureBackend),
}

impl Command {
    pub fn verify(self) -> String {
        if let Command::Verify(secure_backend) = self {
            let storage = secure_storage(secure_backend);
            if !storage.available() {
                return "Unable to access secure storage, check credentials and try again".into();
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

            buffer
        } else {
            panic!("Expected Command::Verify");
        }
    }

    fn write_key(storage: &dyn Storage, buffer: &mut String, key: &str) {
        let value = storage
            .get_public_key(key)
            .map(|c| c.public_key.to_string())
            .unwrap_or_else(|_| "None".into());
        writeln!(buffer, "{} - {}", key, value).unwrap();
    }

    fn write_u64(storage: &dyn Storage, buffer: &mut String, key: &str) {
        let value = storage
            .get(key)
            .and_then(|c| c.value.u64())
            .map(|c| c.to_string())
            .unwrap_or_else(|_| "None".into());
        writeln!(buffer, "{} - {}", key, value).unwrap();
    }

    fn write_waypoint(storage: &dyn Storage, buffer: &mut String, key: &str) {
        let value = if let Ok(value) = storage.get(key).and_then(|c| c.value.string()) {
            if value.is_empty() {
                "empty".into()
            } else {
                Waypoint::from_str(&value)
                    .map(|c| c.to_string())
                    .unwrap_or_else(|_| "Invalid waypoint".into())
            }
        } else {
            "None".into()
        };

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

fn secure_storage(config: SecureBackend) -> Box<dyn Storage> {
    Box::new(VaultStorage::new(
        config.host,
        config.token,
        config.namespace,
    ))
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use libra_secure_storage::{Policy, Value};

    const VAULT_HOST: &str = "http://localhost:8200";
    const VAULT_ROOT_TOKEN: &str = "root_token";

    #[test]
    #[ignore]
    fn test_verify() {
        let namespace = "verify";

        let mut storage = secure_storage(default_secure_backend(namespace.into()));
        storage.reset_and_clear().unwrap();

        let output = verify(namespace).split("None").count();
        assert_eq!(output, 10); // 9 None results in 9 splits

        initialize_storage(default_secure_backend(namespace.into()));

        let output = verify(namespace).split("None").count();
        assert_eq!(output, 1); // 0 None results in 1 split
    }

    fn default_secure_backend(namespace: String) -> SecureBackend {
        SecureBackend {
            host: VAULT_HOST.into(),
            token: VAULT_ROOT_TOKEN.into(),
            namespace: Some(namespace),
        }
    }

    fn initialize_storage(config: SecureBackend) {
        let mut storage = secure_storage(config);
        let policy = Policy::public();
        storage.reset_and_clear().unwrap();

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
    }

    fn verify(namespace: &str) -> String {
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
