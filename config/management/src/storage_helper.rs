// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{error::Error, Command};
use libra_crypto::ed25519::Ed25519PublicKey;
use libra_global_constants::{
    ASSOCIATION_KEY, CONSENSUS_KEY, EPOCH, EXECUTION_KEY, FULLNODE_NETWORK_KEY, LAST_VOTED_ROUND,
    OPERATOR_KEY, OWNER_KEY, PREFERRED_ROUND, VALIDATOR_NETWORK_KEY, WAYPOINT,
};
use libra_network_address::NetworkAddress;
use libra_secure_storage::{
    CryptoStorage, KVStorage, NamespacedStorage, OnDiskStorage, Storage, Value,
};
use libra_types::{chain_id::ChainId, transaction::Transaction, waypoint::Waypoint};
use std::{fs::File, path::Path};
use structopt::StructOpt;

pub struct StorageHelper {
    temppath: libra_temppath::TempPath,
}

impl StorageHelper {
    pub fn new() -> Self {
        let temppath = libra_temppath::TempPath::new();
        temppath.create_as_file().unwrap();
        File::create(temppath.path()).unwrap();
        Self { temppath }
    }

    pub fn storage(&self, namespace: String) -> Storage {
        let storage = OnDiskStorage::new(self.temppath.path().to_path_buf());
        Storage::from(NamespacedStorage::new(Box::new(storage), namespace))
    }

    pub fn path(&self) -> &Path {
        self.temppath.path()
    }

    pub fn path_string(&self) -> &str {
        self.temppath.path().to_str().unwrap()
    }

    pub fn initialize(&self, namespace: String) {
        let mut storage = self.storage(namespace);

        // Initialize all keys in storage
        storage.create_key(ASSOCIATION_KEY).unwrap();
        storage.create_key(CONSENSUS_KEY).unwrap();
        storage.create_key(EXECUTION_KEY).unwrap();
        storage.create_key(FULLNODE_NETWORK_KEY).unwrap();
        storage.create_key(OWNER_KEY).unwrap();
        storage.create_key(OPERATOR_KEY).unwrap();
        storage.create_key(VALIDATOR_NETWORK_KEY).unwrap();

        // Initialize all other data in storage
        storage.set(EPOCH, Value::U64(0)).unwrap();
        storage.set(LAST_VOTED_ROUND, Value::U64(0)).unwrap();
        storage.set(PREFERRED_ROUND, Value::U64(0)).unwrap();
        storage.set(WAYPOINT, Value::String("".into())).unwrap();
    }

    pub fn association_key(
        &self,
        validator_ns: &str,
        shared_ns: &str,
    ) -> Result<Ed25519PublicKey, Error> {
        let args = format!(
            "
                management
                association-key
                --validator-backend backend={backend};\
                    path={path};\
                    namespace={validator_ns}
                --shared-backend backend={backend};\
                    path={path};\
                    namespace={shared_ns}\
            ",
            backend = crate::secure_backend::DISK,
            path = self.path_string(),
            validator_ns = validator_ns,
            shared_ns = shared_ns,
        );

        let command = Command::from_iter(args.split_whitespace());
        command.association_key()
    }

    pub fn create_waypoint(&self, validator_ns: &str) -> Result<Waypoint, Error> {
        let args = format!(
            "
                management
                create-waypoint
                --shared-backend backend={backend};\
                    path={path}
                --validator-backend backend={backend};\
                    path={path};\
                    namespace={validator_ns}\
            ",
            backend = crate::secure_backend::DISK,
            path = self.path_string(),
            validator_ns = validator_ns,
        );

        let command = Command::from_iter(args.split_whitespace());
        command.create_waypoint()
    }

    pub fn genesis(&self, genesis_path: &Path) -> Result<Transaction, Error> {
        let args = format!(
            "
                management
                genesis
                --shared-backend backend={backend};\
                    path={path}
                --path {genesis_path}
            ",
            backend = crate::secure_backend::DISK,
            path = self.path_string(),
            genesis_path = genesis_path.to_str().expect("Unable to parse genesis_path"),
        );

        let command = Command::from_iter(args.split_whitespace());
        command.genesis()
    }

    pub fn insert_waypoint(&self, validator_ns: &str, shared_ns: &str) -> Result<Waypoint, Error> {
        let args = format!(
            "
                management
                insert-waypoint
                --validator-backend backend={backend};\
                    path={path};\
                    namespace={validator_ns}
                --shared-backend backend={backend};\
                    path={path};\
                    namespace={shared_ns}
            ",
            backend = crate::secure_backend::DISK,
            path = self.path_string(),
            validator_ns = validator_ns,
            shared_ns = shared_ns,
        );

        let command = Command::from_iter(args.split_whitespace());
        command.insert_waypoint()
    }

    pub fn operator_key(
        &self,
        validator_ns: &str,
        shared_ns: &str,
    ) -> Result<Ed25519PublicKey, Error> {
        let args = format!(
            "
                management
                operator-key
                --validator-backend backend={backend};\
                    path={path};\
                    namespace={validator_ns}
                --shared-backend backend={backend};\
                    path={path};\
                    namespace={shared_ns}\
            ",
            backend = crate::secure_backend::DISK,
            path = self.path_string(),
            validator_ns = validator_ns,
            shared_ns = shared_ns,
        );

        let command = Command::from_iter(args.split_whitespace());
        command.operator_key()
    }

    pub fn owner_key(
        &self,
        validator_ns: &str,
        shared_ns: &str,
    ) -> Result<Ed25519PublicKey, Error> {
        let args = format!(
            "
                management
                owner-key
                --validator-backend backend={backend};\
                    path={path};\
                    namespace={validator_ns}
                --shared-backend backend={backend};\
                    path={path};\
                    namespace={shared_ns}\
            ",
            backend = crate::secure_backend::DISK,
            path = self.path_string(),
            validator_ns = validator_ns,
            shared_ns = shared_ns,
        );

        let command = Command::from_iter(args.split_whitespace());
        command.owner_key()
    }

    #[cfg(test)]
    pub fn set_layout(&self, path: &str, namespace: &str) -> Result<crate::layout::Layout, Error> {
        let args = format!(
            "
                management
                set-layout
                --path {path}
                --shared-backend backend={backend};\
                    path={storage_path};\
                    namespace={ns}
            ",
            path = path,
            backend = crate::secure_backend::DISK,
            storage_path = self.path_string(),
            ns = namespace,
        );

        let command = Command::from_iter(args.split_whitespace());
        command.set_layout()
    }

    pub fn set_operator(
        &self,
        operator_name: &str,
        validator_ns: &str,
        shared_ns: &str,
    ) -> Result<String, Error> {
        let args = format!(
            "
                management
                set-operator
                --operator-name {operator_name}
                --validator-backend backend={backend};\
                    path={path};\
                    namespace={validator_ns}
                --shared-backend backend={backend};\
                    path={path};\
                    namespace={shared_ns}\
            ",
            operator_name = operator_name,
            backend = crate::secure_backend::DISK,
            path = self.path_string(),
            validator_ns = validator_ns,
            shared_ns = shared_ns,
        );

        let command = Command::from_iter(args.split_whitespace());
        command.set_operator()
    }

    pub fn validator_config(
        &self,
        owner_name: &str,
        validator_address: NetworkAddress,
        fullnode_address: NetworkAddress,
        chain_id: ChainId,
        validator_ns: &str,
        shared_ns: &str,
    ) -> Result<Transaction, Error> {
        let args = format!(
            "
                management
                validator-config
                --owner-name {owner_name}
                --validator-address {validator_address}
                --fullnode-address {fullnode_address}
                --chain-id {chain_id}
                --validator-backend backend={backend};\
                    path={path};\
                    namespace={validator_ns}
                --shared-backend backend={backend};\
                    path={path};\
                    namespace={shared_ns}\
            ",
            owner_name = owner_name,
            validator_address = validator_address,
            fullnode_address = fullnode_address,
            chain_id = chain_id.id(),
            backend = crate::secure_backend::DISK,
            path = self.path_string(),
            validator_ns = validator_ns,
            shared_ns = shared_ns,
        );

        let command = Command::from_iter(args.split_whitespace());
        command.validator_config()
    }

    #[cfg(test)]
    pub fn verify(&self, namespace: &str) -> Result<String, Error> {
        let args = format!(
            "
                management
                verify
                --validator-backend backend={backend};\
                    path={path};\
                    namespace={ns}
            ",
            backend = crate::secure_backend::DISK,
            path = self.path_string(),
            ns = namespace,
        );

        let command = Command::from_iter(args.split_whitespace());
        command.verify()
    }

    pub fn verify_genesis(&self, namespace: &str, genesis_path: &Path) -> Result<String, Error> {
        let args = format!(
            "
                management
                verify
                --validator-backend backend={backend};\
                    path={path};\
                    namespace={ns}
                --genesis-path {genesis_path}
            ",
            backend = crate::secure_backend::DISK,
            path = self.path_string(),
            ns = namespace,
            genesis_path = genesis_path.to_str().expect("Unable to parse genesis_path"),
        );

        let command = Command::from_iter(args.split_whitespace());
        command.verify()
    }
}
