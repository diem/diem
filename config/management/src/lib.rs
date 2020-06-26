// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

mod error;
mod genesis;
mod json_rpc;
mod key;
mod layout;
mod secure_backend;
mod validator_config;
mod verify;
mod waypoint;

#[cfg(test)]
mod smoke_test;

#[cfg(test)]
mod storage_helper;

use crate::{error::Error, layout::SetLayout, secure_backend::SecureBackend};
use libra_crypto::ed25519::Ed25519PublicKey;
use libra_types::{transaction::Transaction, waypoint::Waypoint};
use structopt::StructOpt;

pub mod constants {
    use libra_types::account_config::LBR_NAME;
    pub const COMMON_NS: &str = "common";
    pub const LAYOUT: &str = "layout";
    pub const VALIDATOR_CONFIG: &str = "validator_config";

    pub const GAS_UNIT_PRICE: u64 = 0;
    pub const MAX_GAS_AMOUNT: u64 = 1_000_000;
    pub const GAS_CURRENCY_CODE: &str = LBR_NAME;
    pub const TXN_EXPIRATION_SECS: u64 = 3600;
}

#[derive(Debug, StructOpt)]
#[structopt(about = "Tool used to manage Libra Validators")]
pub enum Command {
    #[structopt(about = "Submits an Ed25519PublicKey for the association")]
    AssociationKey(crate::key::AssociationKey),
    #[structopt(about = "Create a waypoint and optionally place it in a store")]
    CreateWaypoint(crate::waypoint::CreateWaypoint),
    #[structopt(about = "Retrieves data from a store to produce genesis")]
    Genesis(crate::genesis::Genesis),
    #[structopt(about = "Insert a waypoint")]
    InsertWaypoint(crate::waypoint::InsertWaypoint),
    #[structopt(about = "Submits an Ed25519PublicKey for the operator")]
    OperatorKey(crate::key::OperatorKey),
    #[structopt(about = "Submits an Ed25519PublicKey for the owner")]
    OwnerKey(crate::key::OwnerKey),
    #[structopt(about = "Read account state from JSON-RPC endpoint")]
    ReadAccountState(crate::json_rpc::ReadAccountState),
    #[structopt(about = "Submit a transaction to the blockchain")]
    SubmitTransaction(crate::json_rpc::SubmitTransaction),
    #[structopt(about = "Submits a Layout doc to a shared storage")]
    SetLayout(SetLayout),
    #[structopt(about = "Constructs and signs a ValidatorConfig")]
    ValidatorConfig(crate::validator_config::ValidatorConfig),
    #[structopt(about = "Verifies and prints the current configuration state")]
    Verify(crate::verify::Verify),
}

#[derive(Debug, PartialEq)]
pub enum CommandName {
    AssociationKey,
    CreateWaypoint,
    Genesis,
    InsertWaypoint,
    OperatorKey,
    OwnerKey,
    ReadAccountState,
    SetLayout,
    SubmitTransaction,
    ValidatorConfig,
    Verify,
}

impl From<&Command> for CommandName {
    fn from(command: &Command) -> Self {
        match command {
            Command::AssociationKey(_) => CommandName::AssociationKey,
            Command::CreateWaypoint(_) => CommandName::CreateWaypoint,
            Command::Genesis(_) => CommandName::Genesis,
            Command::InsertWaypoint(_) => CommandName::InsertWaypoint,
            Command::OperatorKey(_) => CommandName::OperatorKey,
            Command::OwnerKey(_) => CommandName::OwnerKey,
            Command::ReadAccountState(_) => CommandName::ReadAccountState,
            Command::SetLayout(_) => CommandName::SetLayout,
            Command::SubmitTransaction(_) => CommandName::SubmitTransaction,
            Command::ValidatorConfig(_) => CommandName::ValidatorConfig,
            Command::Verify(_) => CommandName::Verify,
        }
    }
}

impl std::fmt::Display for CommandName {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let name = match self {
            CommandName::AssociationKey => "association-key",
            CommandName::CreateWaypoint => "create-waypoint",
            CommandName::Genesis => "genesis",
            CommandName::InsertWaypoint => "insert-waypoint",
            CommandName::OperatorKey => "operator-key",
            CommandName::OwnerKey => "owner-key",
            CommandName::ReadAccountState => "read-account-state",
            CommandName::SetLayout => "set-layout",
            CommandName::SubmitTransaction => "submit-transaction",
            CommandName::ValidatorConfig => "validator-config",
            CommandName::Verify => "verify",
        };
        write!(f, "{}", name)
    }
}

impl Command {
    pub fn execute(self) -> String {
        match &self {
            Command::AssociationKey(_) => self.association_key().unwrap().to_string(),
            Command::CreateWaypoint(_) => self.create_waypoint().unwrap().to_string(),
            Command::Genesis(_) => format!("{:?}", self.genesis().unwrap()),
            Command::InsertWaypoint(_) => self.insert_waypoint().unwrap().to_string(),
            Command::OperatorKey(_) => self.operator_key().unwrap().to_string(),
            Command::OwnerKey(_) => self.owner_key().unwrap().to_string(),
            Command::ReadAccountState(_) => format!("{:?}", self.read_account_state().unwrap()),
            Command::SetLayout(_) => self.set_layout().unwrap().to_string(),
            Command::SubmitTransaction(_) => self
                .submit_transaction()
                .map(|_| "success!")
                .unwrap()
                .to_string(),
            Command::ValidatorConfig(_) => format!("{:?}", self.validator_config().unwrap()),
            Command::Verify(_) => self.verify().unwrap(),
        }
    }

    pub fn association_key(self) -> Result<Ed25519PublicKey, Error> {
        match self {
            Command::AssociationKey(association_key) => association_key.execute(),
            _ => Err(self.unexpected_command(CommandName::AssociationKey)),
        }
    }

    pub fn create_waypoint(self) -> Result<Waypoint, Error> {
        match self {
            Command::CreateWaypoint(create_waypoint) => create_waypoint.execute(),
            _ => Err(self.unexpected_command(CommandName::CreateWaypoint)),
        }
    }

    pub fn genesis(self) -> Result<Transaction, Error> {
        match self {
            Command::Genesis(genesis) => genesis.execute(),
            _ => Err(self.unexpected_command(CommandName::Genesis)),
        }
    }

    pub fn insert_waypoint(self) -> Result<Waypoint, Error> {
        match self {
            Command::InsertWaypoint(insert_waypoint) => insert_waypoint.execute(),
            _ => Err(self.unexpected_command(CommandName::InsertWaypoint)),
        }
    }

    pub fn operator_key(self) -> Result<Ed25519PublicKey, Error> {
        match self {
            Command::OperatorKey(operator_key) => operator_key.execute(),
            _ => Err(self.unexpected_command(CommandName::OperatorKey)),
        }
    }

    pub fn owner_key(self) -> Result<Ed25519PublicKey, Error> {
        match self {
            Command::OwnerKey(owner_key) => owner_key.execute(),
            _ => Err(self.unexpected_command(CommandName::OwnerKey)),
        }
    }

    pub fn read_account_state(self) -> Result<libra_types::account_state::AccountState, Error> {
        match self {
            Command::ReadAccountState(read_account_state) => read_account_state.execute(),
            _ => Err(self.unexpected_command(CommandName::ReadAccountState)),
        }
    }

    pub fn set_layout(self) -> Result<crate::layout::Layout, Error> {
        match self {
            Command::SetLayout(set_layout) => set_layout.execute(),
            _ => Err(self.unexpected_command(CommandName::SetLayout)),
        }
    }

    pub fn submit_transaction(self) -> Result<(), Error> {
        match self {
            Command::SubmitTransaction(submit_transaction) => submit_transaction.execute(),
            _ => Err(self.unexpected_command(CommandName::SubmitTransaction)),
        }
    }

    pub fn validator_config(self) -> Result<Transaction, Error> {
        match self {
            Command::ValidatorConfig(config) => config.execute(),
            _ => Err(self.unexpected_command(CommandName::ValidatorConfig)),
        }
    }

    pub fn verify(self) -> Result<String, Error> {
        match self {
            Command::Verify(verify) => verify.execute(),
            _ => Err(self.unexpected_command(CommandName::Verify)),
        }
    }

    fn unexpected_command(self, expected: CommandName) -> Error {
        Error::UnexpectedCommand(expected, CommandName::from(&self))
    }
}

#[derive(Debug, StructOpt)]
pub struct SecureBackends {
    /// The local secure backend, this is the source of data. Secure
    /// backends are represented as a semi-colon deliminted key value
    /// pair: "k0=v0;k1=v1;...".  The current supported formats are:
    ///     Vault: "backend=vault;server=URL;token=PATH_TO_TOKEN"
    ///         an optional namespace: "namespace=NAMESPACE"
    ///         an optional server certificate: "ca_certificate=PATH_TO_CERT"
    ///     GitHub: "backend=github;repository_owner=REPOSITORY_OWNER;repository=REPOSITORY;token=PATH_TO_TOKEN"
    ///         an optional namespace: "namespace=NAMESPACE"
    ///     InMemory: "backend=memory"
    ///     OnDisk: "backend=disk;path=LOCAL_PATH"
    #[structopt(long, verbatim_doc_comment)]
    local: SecureBackend,
    /// The remote secure backend, this is where data is stored. See
    /// the comments for the local backend for usage.
    #[structopt(long)]
    remote: Option<SecureBackend>,
}

#[derive(Debug, StructOpt)]
pub struct SingleBackend {
    /// The secure backend. Secure backends are represented as a semi-colon
    /// deliminted key value pair: "k0=v0;k1=v1;...".
    /// The current supported formats are:
    ///     Vault: "backend=vault;server=URL;token=PATH_TO_TOKEN"
    ///         an optional namespace: "namespace=NAMESPACE"
    ///         an optional server certificate: "ca_certificate=PATH_TO_CERT"
    ///     GitHub: "backend=github;repository_owner=REPOSITORY_OWNER;repository=REPOSITORY;token=PATH_TO_TOKEN"
    ///         an optional namespace: "namespace=NAMESPACE"
    ///     InMemory: "backend=memory"
    ///     OnDisk: "backend=disk;path=LOCAL_PATH"
    #[structopt(long, verbatim_doc_comment)]
    pub backend: SecureBackend,
}

/// These tests depends on running Vault, which can be done by using the provided docker run script
/// in `docker/vault/run.sh`.
/// Note: Some of these tests may fail if you run them too quickly one after another due to data
/// sychronization issues within Vault. It would seem the only way to fix it would be to restart
/// the Vault service between runs.
#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::storage_helper::StorageHelper;
    use libra_secure_storage::{CryptoStorage, KVStorage};
    use libra_types::account_address::AccountAddress;
    use std::{
        fs::File,
        io::{Read, Write},
    };

    #[test]
    fn test_end_to_end() {
        let helper = StorageHelper::new();

        // Each identity works in their own namespace
        // Alice, Bob, and Carol are operators, implicitly mapped 1:1 with owners.
        // Dave is the association.
        // Each user will upload their contents to *_ns + "shared"
        // Common is used by the technical staff for coordination.
        let alice_ns = "alice";
        let bob_ns = "bob";
        let carol_ns = "carol";
        let dave_ns = "dave";
        let shared = "_shared";

        // Step 1) Define and upload the layout specifying which identities have which roles. This
        // is uplaoded to the common namespace.

        // Note: owners are irrelevant currently
        let layout_text = "\
            operators = [\"alice_shared\", \"bob_shared\", \"carol_shared\"]\n\
            owners = []\n\
            association = [\"dave_shared\"]\n\
        ";

        let temppath = libra_temppath::TempPath::new();
        temppath.create_as_file().unwrap();
        let mut file = File::create(temppath.path()).unwrap();
        file.write_all(&layout_text.to_string().into_bytes())
            .unwrap();
        file.sync_all().unwrap();

        helper
            .set_layout(
                temppath.path().to_str().unwrap(),
                crate::constants::COMMON_NS,
            )
            .unwrap();

        // Step 2) Upload the association key:

        helper.initialize(dave_ns.into());
        helper
            .association_key(dave_ns, &(dave_ns.to_string() + shared))
            .unwrap();

        // Step 3) Upload each operators key and then a signed transaction:

        for ns in [alice_ns, bob_ns, carol_ns].iter() {
            helper.initialize((*ns).to_string());
            helper
                .operator_key(ns, &((*ns).to_string() + shared))
                .unwrap();

            helper
                .validator_config(
                    AccountAddress::random(),
                    "/ip4/0.0.0.0/tcp/6180".parse().unwrap(),
                    "/ip4/0.0.0.0/tcp/6180".parse().unwrap(),
                    ns,
                    &((*ns).to_string() + shared),
                )
                .unwrap();
        }

        // Step 4) Produce genesis

        let genesis_path = libra_temppath::TempPath::new();
        genesis_path.create_as_file().unwrap();
        helper.genesis(genesis_path.path()).unwrap();
        let mut file = File::open(genesis_path.path()).unwrap();
        let mut contents = Vec::new();
        assert!(contents.is_empty());
        file.read_to_end(&mut contents).unwrap();
        assert!(!contents.is_empty());
    }

    #[test]
    fn test_set_layout() {
        let helper = StorageHelper::new();
        let namespace = "set_layout";

        let temppath = libra_temppath::TempPath::new();
        helper
            .set_layout(temppath.path().to_str().unwrap(), namespace)
            .unwrap_err();

        temppath.create_as_file().unwrap();
        let mut file = File::create(temppath.path()).unwrap();
        let layout_text = "\
            operators = [\"alice\", \"bob\"]\n\
            owners = [\"carol\"]\n\
            association = [\"dave\"]\n\
        ";
        file.write_all(&layout_text.to_string().into_bytes())
            .unwrap();
        file.sync_all().unwrap();

        helper
            .set_layout(temppath.path().to_str().unwrap(), namespace)
            .unwrap();
        let storage = helper.storage(namespace.into());
        let stored_layout = storage
            .get(constants::LAYOUT)
            .unwrap()
            .value
            .string()
            .unwrap();
        assert_eq!(layout_text, stored_layout);
    }

    #[test]
    fn test_validator_config() {
        let helper = StorageHelper::new();
        let local_ns = "local_validator_config";
        let remote_ns = "remote_validator_config";

        helper.initialize(local_ns.into());

        let local_txn = helper
            .validator_config(
                AccountAddress::random(),
                "/ip4/0.0.0.0/tcp/6180".parse().unwrap(),
                "/ip4/0.0.0.0/tcp/6180".parse().unwrap(),
                local_ns,
                remote_ns,
            )
            .unwrap();

        let remote = helper.storage(remote_ns.into());
        let remote_txn = remote.get(constants::VALIDATOR_CONFIG).unwrap().value;
        let remote_txn = remote_txn.transaction().unwrap();

        assert_eq!(local_txn, remote_txn);
    }

    #[test]
    fn test_verify() {
        let helper = StorageHelper::new();
        let namespace = "verify";

        let output = helper
            .verify(namespace)
            .unwrap()
            .split("Key not set")
            .count();
        // 11 KeyNotSet results in 12 splits
        assert_eq!(output, 12);

        helper.initialize(namespace.into());

        let output = helper
            .verify(namespace)
            .unwrap()
            .split("Key not set")
            .count();
        // 2 KeyNotSet results in 3 split (the accounts aren't initialized via initialize)
        assert_eq!(output, 3);
    }

    #[test]
    fn test_owner_key() {
        test_key(libra_global_constants::OWNER_KEY, StorageHelper::owner_key);
    }

    #[test]
    fn test_operator_key() {
        test_key(
            libra_global_constants::OPERATOR_KEY,
            StorageHelper::operator_key,
        );
    }

    fn test_key(
        key_name: &str,
        op: fn(&StorageHelper, &str, &str) -> Result<Ed25519PublicKey, Error>,
    ) {
        let helper = StorageHelper::new();
        let local_ns = format!("local_{}_key", key_name);
        let remote_ns = format!("remote_{}_key", key_name);

        op(&helper, &local_ns, &remote_ns).unwrap_err();

        helper.initialize(local_ns.clone());
        let local = helper.storage(local_ns.clone());
        let local_key = local.get_public_key(key_name).unwrap().public_key;

        let output_key = op(&helper, &local_ns, &remote_ns).unwrap();
        let remote = helper.storage(remote_ns);
        let remote_key = remote
            .get(key_name)
            .unwrap()
            .value
            .ed25519_public_key()
            .unwrap();

        assert_eq!(local_key, output_key);
        assert_eq!(local_key, remote_key);
    }
}
