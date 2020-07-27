// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0
use libra_crypto::ed25519::Ed25519PublicKey;
use libra_management::error::Error;
use libra_types::{transaction::Transaction, waypoint::Waypoint};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(about = "Tool used for genesis")]
pub enum Command {
    #[structopt(about = "Create a waypoint and place it in a store")]
    CreateAndInsertWaypoint(crate::waypoint::CreateAndInsertWaypoint),
    #[structopt(about = "Create a waypoint")]
    CreateWaypoint(crate::waypoint::CreateWaypoint),
    #[structopt(about = "Retrieves data from a store to produce genesis")]
    Genesis(crate::genesis::Genesis),
    #[structopt(about = "Submits an Ed25519PublicKey for the libra root")]
    LibraRootKey(crate::key::LibraRootKey),
    #[structopt(about = "Submits an Ed25519PublicKey for the operator")]
    OperatorKey(crate::key::OperatorKey),
    #[structopt(about = "Submits an Ed25519PublicKey for the owner")]
    OwnerKey(crate::key::OwnerKey),
    #[structopt(about = "Submits a Layout doc to a shared storage")]
    SetLayout(crate::layout::SetLayout),
    #[structopt(about = "Sets the validator operator chosen by the owner")]
    SetOperator(crate::validator_operator::ValidatorOperator),
    #[structopt(about = "Constructs and signs a ValidatorConfig")]
    ValidatorConfig(crate::validator_config::ValidatorConfig),
    #[structopt(about = "Verifies and prints the current configuration state")]
    Verify(crate::verify::Verify),
}

#[derive(Debug, PartialEq)]
pub enum CommandName {
    CreateAndInsertWaypoint,
    CreateWaypoint,
    Genesis,
    LibraRootKey,
    OperatorKey,
    OwnerKey,
    SetLayout,
    SetOperator,
    ValidatorConfig,
    Verify,
}

impl From<&Command> for CommandName {
    fn from(command: &Command) -> Self {
        match command {
            Command::CreateAndInsertWaypoint(_) => CommandName::CreateAndInsertWaypoint,
            Command::CreateWaypoint(_) => CommandName::CreateWaypoint,
            Command::Genesis(_) => CommandName::Genesis,
            Command::LibraRootKey(_) => CommandName::LibraRootKey,
            Command::OperatorKey(_) => CommandName::OperatorKey,
            Command::OwnerKey(_) => CommandName::OwnerKey,
            Command::SetLayout(_) => CommandName::SetLayout,
            Command::SetOperator(_) => CommandName::SetOperator,
            Command::ValidatorConfig(_) => CommandName::ValidatorConfig,
            Command::Verify(_) => CommandName::Verify,
        }
    }
}

impl std::fmt::Display for CommandName {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let name = match self {
            CommandName::CreateAndInsertWaypoint => "create-and-insert-waypoint",
            CommandName::CreateWaypoint => "create-waypoint",
            CommandName::Genesis => "genesis",
            CommandName::LibraRootKey => "libra-root-key",
            CommandName::OperatorKey => "operator-key",
            CommandName::OwnerKey => "owner-key",
            CommandName::SetLayout => "set-layout",
            CommandName::SetOperator => "set-operator",
            CommandName::ValidatorConfig => "validator-config",
            CommandName::Verify => "verify",
        };
        write!(f, "{}", name)
    }
}

impl Command {
    pub fn execute(self) -> String {
        match &self {
            Command::CreateAndInsertWaypoint(_) => {
                self.create_and_insert_waypoint().unwrap().to_string()
            }
            Command::CreateWaypoint(_) => self.create_waypoint().unwrap().to_string(),
            Command::Genesis(_) => format!("{:?}", self.genesis().unwrap()),
            Command::LibraRootKey(_) => self.libra_root_key().unwrap().to_string(),
            Command::OperatorKey(_) => self.operator_key().unwrap().to_string(),
            Command::OwnerKey(_) => self.owner_key().unwrap().to_string(),
            Command::SetLayout(_) => self.set_layout().unwrap().to_string(),
            Command::SetOperator(_) => format!("{:?}", self.set_operator().unwrap()),
            Command::ValidatorConfig(_) => format!("{:?}", self.validator_config().unwrap()),
            Command::Verify(_) => self.verify().unwrap(),
        }
    }

    pub fn create_and_insert_waypoint(self) -> Result<Waypoint, Error> {
        match self {
            Command::CreateAndInsertWaypoint(cmd) => cmd.execute(),
            _ => Err(self.unexpected_command(CommandName::CreateAndInsertWaypoint)),
        }
    }

    pub fn create_waypoint(self) -> Result<Waypoint, Error> {
        match self {
            Command::CreateWaypoint(cmd) => cmd.execute(),
            _ => Err(self.unexpected_command(CommandName::CreateWaypoint)),
        }
    }

    pub fn genesis(self) -> Result<Transaction, Error> {
        match self {
            Command::Genesis(cmd) => cmd.execute(),
            _ => Err(self.unexpected_command(CommandName::Genesis)),
        }
    }

    pub fn libra_root_key(self) -> Result<Ed25519PublicKey, Error> {
        match self {
            Command::LibraRootKey(cmd) => cmd.execute(),
            _ => Err(self.unexpected_command(CommandName::LibraRootKey)),
        }
    }

    pub fn operator_key(self) -> Result<Ed25519PublicKey, Error> {
        match self {
            Command::OperatorKey(cmd) => cmd.execute(),
            _ => Err(self.unexpected_command(CommandName::OperatorKey)),
        }
    }

    pub fn owner_key(self) -> Result<Ed25519PublicKey, Error> {
        match self {
            Command::OwnerKey(cmd) => cmd.execute(),
            _ => Err(self.unexpected_command(CommandName::OwnerKey)),
        }
    }

    pub fn set_layout(self) -> Result<crate::layout::Layout, Error> {
        match self {
            Command::SetLayout(cmd) => cmd.execute(),
            _ => Err(self.unexpected_command(CommandName::SetLayout)),
        }
    }

    pub fn set_operator(self) -> Result<String, Error> {
        match self {
            Command::SetOperator(cmd) => cmd.execute(),
            _ => Err(self.unexpected_command(CommandName::SetOperator)),
        }
    }

    pub fn validator_config(self) -> Result<Transaction, Error> {
        match self {
            Command::ValidatorConfig(cmd) => cmd.execute(),
            _ => Err(self.unexpected_command(CommandName::ValidatorConfig)),
        }
    }

    pub fn verify(self) -> Result<String, Error> {
        match self {
            Command::Verify(cmd) => cmd.execute(),
            _ => Err(self.unexpected_command(CommandName::Verify)),
        }
    }

    fn unexpected_command(self, expected: CommandName) -> Error {
        Error::UnexpectedCommand(expected.to_string(), CommandName::from(&self).to_string())
    }
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
    use libra_crypto::{ed25519::Ed25519PrivateKey, PrivateKey, Uniform};
    use libra_global_constants::{OPERATOR_KEY, OWNER_KEY};
    use libra_management::constants;
    use libra_secure_storage::{KVStorage, Value};
    use libra_types::{
        account_address,
        chain_id::ChainId,
        transaction::{TransactionArgument, TransactionPayload},
    };
    use std::{
        fs::File,
        io::{Read, Write},
    };

    #[test]
    fn test_end_to_end() {
        let helper = StorageHelper::new();

        // Each identity works in their own namespace
        // Alice, Bob, and Carol are owners.
        // Operator_Alice, Operator_Bob and Operator_Carol are operators.
        // Dave is the libra root.
        // Each user will upload their contents to *_ns + "shared"
        // Common is used by the technical staff for coordination.
        let alice_ns = "alice";
        let bob_ns = "bob";
        let carol_ns = "carol";
        let operator_alice_ns = "operator_alice";
        let operator_bob_ns = "operator_bob";
        let operator_carol_ns = "operator_carol";
        let dave_ns = "dave";
        let shared = "_shared";

        // Step 1) Define and upload the layout specifying which identities have which roles. This
        // is uploaded to the common namespace:
        let layout_text = "\
            operators = [\"operator_alice_shared\", \"operator_bob_shared\", \"operator_carol_shared\"]\n\
            owners = [\"alice_shared\", \"bob_shared\", \"carol_shared\"]\n\
            libra_root = [\"dave_shared\"]\n\
        ";

        let temppath = libra_temppath::TempPath::new();
        temppath.create_as_file().unwrap();
        let mut file = File::create(temppath.path()).unwrap();
        file.write_all(&layout_text.to_string().into_bytes())
            .unwrap();
        file.sync_all().unwrap();

        helper
            .set_layout(temppath.path().to_str().unwrap(), constants::COMMON_NS)
            .unwrap();

        // Step 2) Upload the libra root key:
        helper.initialize(dave_ns.into());
        helper
            .libra_root_key(dave_ns, &(dave_ns.to_string() + shared))
            .unwrap();

        // Step 3) Upload each owner key:
        for ns in [alice_ns, bob_ns, carol_ns].iter() {
            let ns = (*ns).to_string();
            let ns_shared = (*ns).to_string() + shared;

            helper.initialize(ns.clone());
            helper.owner_key(&ns, &ns_shared).unwrap();
        }

        // Step 4) Upload each operator key:
        for ns in [operator_alice_ns, operator_bob_ns, operator_carol_ns].iter() {
            let ns = (*ns).to_string();
            let ns_shared = (*ns).to_string() + shared;

            helper.initialize(ns.clone());
            helper.operator_key(&ns, &ns_shared).unwrap();
        }

        // Step 5) Set the operator for each owner:
        for ns in [alice_ns, bob_ns, carol_ns].iter() {
            let ns_shared = (*ns).to_string() + shared;

            let operator_name = format!("operator_{}", ns_shared);
            helper.set_operator(&operator_name, &ns_shared).unwrap();
        }

        // Step 6) Upload a signed validator config transaction for each operator:
        for ns in [operator_alice_ns, operator_bob_ns, operator_carol_ns].iter() {
            let ns = (*ns).to_string();
            let ns_shared = (*ns).to_string() + shared;

            let owner_name: String = (*ns).chars().skip(9).collect(); // Remove "operator_" prefix
            let owner_name = owner_name + shared;
            helper
                .validator_config(
                    &owner_name,
                    "/ip4/0.0.0.0/tcp/6180".parse().unwrap(),
                    "/ip4/0.0.0.0/tcp/6180".parse().unwrap(),
                    ChainId::test(),
                    &ns,
                    &ns_shared,
                )
                .unwrap();
        }

        // Step 7) Produce genesis
        let genesis_path = libra_temppath::TempPath::new();
        genesis_path.create_as_file().unwrap();
        helper.genesis(genesis_path.path()).unwrap();
        let mut file = File::open(genesis_path.path()).unwrap();
        let mut contents = Vec::new();
        assert!(contents.is_empty());
        file.read_to_end(&mut contents).unwrap();
        assert!(!contents.is_empty());

        // Step 9) Verify
        for ns in [operator_alice_ns, operator_bob_ns, operator_carol_ns].iter() {
            helper.create_and_insert_waypoint(ns).unwrap();
            helper.verify_genesis(ns, genesis_path.path()).unwrap();
        }
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
            libra_root = [\"dave\"]\n\
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
        let storage_helper = StorageHelper::new();
        let local_operator_ns = "local";
        let remote_operator_ns = "operator";
        storage_helper.initialize(local_operator_ns.into());

        // Operator uploads key to shared storage and initializes address in local storage
        let operator_key = storage_helper
            .operator_key(local_operator_ns, remote_operator_ns)
            .unwrap();

        // Upload an owner key to the remote storage
        let owner_name = "owner";
        let owner_key = Ed25519PrivateKey::generate_for_testing().public_key();
        let owner_account = account_address::from_public_key(&owner_key);
        let mut shared_storage = storage_helper.storage(owner_name.into());
        shared_storage
            .set(OWNER_KEY, Value::Ed25519PublicKey(owner_key))
            .map_err(|e| Error::StorageWriteError("shared", OWNER_KEY, e.to_string()))
            .unwrap();

        // Operator calls the validator-config command
        let local_config_tx = storage_helper
            .validator_config(
                owner_name,
                "/ip4/0.0.0.0/tcp/6180".parse().unwrap(),
                "/ip4/0.0.0.0/tcp/6180".parse().unwrap(),
                ChainId::test(),
                local_operator_ns,
                remote_operator_ns,
            )
            .unwrap();

        // Verify that a validator config transaction was uploaded to the remote storage
        let shared_storage = storage_helper.storage(remote_operator_ns.into());
        let uploaded_config_tx = shared_storage
            .get(constants::VALIDATOR_CONFIG)
            .unwrap()
            .value
            .transaction()
            .unwrap();
        assert_eq!(local_config_tx, uploaded_config_tx);

        // Verify the transaction sender is the operator account address
        let operator_account = account_address::from_public_key(&operator_key);
        let uploaded_user_transaction = uploaded_config_tx.as_signed_user_txn().unwrap();
        assert_eq!(operator_account, uploaded_user_transaction.sender());

        // Verify the validator config in the transaction has the correct account address
        match uploaded_user_transaction.payload() {
            TransactionPayload::Script(script) => {
                assert_eq!(6, script.args().len());

                match script.args().get(0).unwrap() {
                    TransactionArgument::Address(account_address) => {
                        assert_eq!(&owner_account, account_address);
                    }
                    _ => panic!(
                        "Found an invalid argument type for the validator-config transaction script!"
                    ),
                };
            }
            _ => panic!("Invalid validator-config transaction payload found!"),
        };
    }

    #[test]
    fn test_set_operator() {
        let storage_helper = StorageHelper::new();
        let local_owner_ns = "local";
        let remote_owner_ns = "owner";
        storage_helper.initialize(local_owner_ns.into());

        // Upload an operator key to the remote storage
        let operator_name = "operator";
        let operator_key = Ed25519PrivateKey::generate_for_testing().public_key();
        let mut shared_storage = storage_helper.storage(operator_name.into());
        shared_storage
            .set(OPERATOR_KEY, Value::Ed25519PublicKey(operator_key))
            .map_err(|e| Error::StorageWriteError("shared", OPERATOR_KEY, e.to_string()))
            .unwrap();

        // Owner calls the set-operator command
        let local_operator_name = storage_helper
            .set_operator(operator_name, remote_owner_ns)
            .unwrap();

        // Verify that a file setting the operator was uploaded to the remote storage
        let shared_storage = storage_helper.storage(remote_owner_ns.into());
        let uploaded_operator_name = shared_storage
            .get(constants::VALIDATOR_OPERATOR)
            .unwrap()
            .value
            .string()
            .unwrap();
        assert_eq!(local_operator_name, uploaded_operator_name);
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
}
