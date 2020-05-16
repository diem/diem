// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

mod error;
mod genesis;
mod key;
mod layout;
mod secure_backend;
mod validator_config;
mod verify;

#[cfg(test)]
mod smoke_test;

#[cfg(test)]
mod storage_helper;

use crate::{error::Error, layout::SetLayout, secure_backend::SecureBackend};
use libra_crypto::ed25519::Ed25519PublicKey;
use libra_types::transaction::Transaction;
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
    pub const HANDSHAKE_VERSION: u8 = 0;
}

#[derive(Debug, StructOpt)]
#[structopt(about = "Tool used to manage Libra Validators")]
pub enum Command {
    #[structopt(about = "Submits an Ed25519PublicKey for the association")]
    AssociationKey(crate::key::AssociationKey),
    #[structopt(about = "Retrieves data from a store to produce genesis")]
    Genesis(crate::genesis::Genesis),
    #[structopt(about = "Submits an Ed25519PublicKey for the operator")]
    OperatorKey(crate::key::OperatorKey),
    #[structopt(about = "Submits an Ed25519PublicKey for the owner")]
    OwnerKey(crate::key::OwnerKey),
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
    Genesis,
    OperatorKey,
    OwnerKey,
    SetLayout,
    ValidatorConfig,
    Verify,
}

impl From<&Command> for CommandName {
    fn from(command: &Command) -> Self {
        match command {
            Command::AssociationKey(_) => CommandName::AssociationKey,
            Command::Genesis(_) => CommandName::Genesis,
            Command::OperatorKey(_) => CommandName::OperatorKey,
            Command::OwnerKey(_) => CommandName::OwnerKey,
            Command::SetLayout(_) => CommandName::SetLayout,
            Command::ValidatorConfig(_) => CommandName::ValidatorConfig,
            Command::Verify(_) => CommandName::Verify,
        }
    }
}

impl std::fmt::Display for CommandName {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let name = match self {
            CommandName::AssociationKey => "association-key",
            CommandName::Genesis => "genesis",
            CommandName::OperatorKey => "operator-key",
            CommandName::OwnerKey => "owner-key",
            CommandName::SetLayout => "set-layout",
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
            Command::Genesis(_) => format!("{:?}", self.genesis().unwrap()),
            Command::OperatorKey(_) => self.operator_key().unwrap().to_string(),
            Command::OwnerKey(_) => self.owner_key().unwrap().to_string(),
            Command::SetLayout(_) => self.set_layout().unwrap().to_string(),
            Command::ValidatorConfig(_) => format!("{:?}", self.validator_config().unwrap()),
            Command::Verify(_) => self.verify().unwrap(),
        }
    }

    pub fn association_key(self) -> Result<Ed25519PublicKey, Error> {
        if let Command::AssociationKey(association_key) = self {
            association_key.execute()
        } else {
            Err(Error::UnexpectedCommand(
                CommandName::AssociationKey,
                CommandName::from(&self),
            ))
        }
    }

    pub fn genesis(self) -> Result<Transaction, Error> {
        if let Command::Genesis(genesis) = self {
            genesis.execute()
        } else {
            Err(Error::UnexpectedCommand(
                CommandName::Genesis,
                CommandName::from(&self),
            ))
        }
    }

    pub fn operator_key(self) -> Result<Ed25519PublicKey, Error> {
        if let Command::OperatorKey(operator_key) = self {
            operator_key.execute()
        } else {
            Err(Error::UnexpectedCommand(
                CommandName::OperatorKey,
                CommandName::from(&self),
            ))
        }
    }

    pub fn owner_key(self) -> Result<Ed25519PublicKey, Error> {
        if let Command::OwnerKey(owner_key) = self {
            owner_key.execute()
        } else {
            Err(Error::UnexpectedCommand(
                CommandName::OwnerKey,
                CommandName::from(&self),
            ))
        }
    }

    pub fn set_layout(self) -> Result<crate::layout::Layout, Error> {
        if let Command::SetLayout(set_layout) = self {
            set_layout.execute()
        } else {
            Err(Error::UnexpectedCommand(
                CommandName::SetLayout,
                CommandName::from(&self),
            ))
        }
    }

    pub fn validator_config(self) -> Result<Transaction, Error> {
        if let Command::ValidatorConfig(config) = self {
            config.execute()
        } else {
            Err(Error::UnexpectedCommand(
                CommandName::ValidatorConfig,
                CommandName::from(&self),
            ))
        }
    }

    pub fn verify(self) -> Result<String, Error> {
        if let Command::Verify(verify) = self {
            verify.execute()
        } else {
            Err(Error::UnexpectedCommand(
                CommandName::Verify,
                CommandName::from(&self),
            ))
        }
    }
}

#[derive(Debug, StructOpt)]
pub struct SecureBackends {
    /// The local secure backend, this is the source of data. Secure
    /// backends are represented as a semi-colon deliminted key value
    /// pair: "k0=v0;k1=v1;...".  The current supported formats are:
    ///     Vault: "backend=vault;server=URL;token=TOKEN"
    ///         vault has an optional namespace: "namespace=NAMESPACE"
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
    ///     Vault: "backend=vault;server=URL;token=TOKEN"
    ///         vault has an optional namespace: "namespace=NAMESPACE"
    ///     InMemory: "backend=memory"
    ///     OnDisk: "backend=disk;path=LOCAL_PATH"
    #[structopt(long, verbatim_doc_comment)]
    backend: SecureBackend,
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
    use libra_types::account_address::AccountAddress;
    use std::{fs::File, io::Write};

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

        helper.genesis().unwrap();
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

        let output = helper.verify(namespace).unwrap().split("KeyNotSet").count();
        assert_eq!(output, 10); // 9 KeyNotSet results in 9 splits

        helper.initialize(namespace.into());

        let output = helper.verify(namespace).unwrap().split("KeyNotSet").count();
        assert_eq!(output, 1); // 0 KeyNotSet results in 1 split
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
