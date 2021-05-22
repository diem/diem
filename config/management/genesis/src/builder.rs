// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::layout::Layout;
use anyhow::Result;
use diem_crypto::ed25519::{Ed25519PrivateKey, Ed25519PublicKey};
use diem_global_constants::{
    CONSENSUS_KEY, DIEM_ROOT_KEY, EXECUTION_KEY, FULLNODE_NETWORK_KEY, OPERATOR_KEY, OWNER_KEY,
    SAFETY_DATA, TREASURY_COMPLIANCE_KEY, VALIDATOR_NETWORK_KEY, WAYPOINT,
};
use diem_management::{config::ConfigPath, constants, error::Error, secure_backend::SharedBackend};
use diem_secure_storage::{CryptoStorage, KVStorage, Storage};
use diem_types::chain_id::ChainId;
use serde::{de::DeserializeOwned, Serialize};

pub struct GenesisBuilder<S> {
    storage: S,
}

impl<S> GenesisBuilder<S> {
    fn new(storage: S) -> Self {
        Self { storage }
    }
}

impl<S: KVStorage> GenesisBuilder<S> {
    fn get_with_namespace<T: DeserializeOwned>(&self, namespace: &str, key: &str) -> Result<T> {
        let key = format!("{}/{}", namespace, key);
        self.storage
            .get::<T>(&key)
            .map(|v| v.value)
            .map_err(Into::into)
    }

    fn set_with_namespace<T: Serialize>(
        &mut self,
        namespace: &str,
        key: &str,
        value: T,
    ) -> Result<()> {
        let key = format!("{}/{}", namespace, key);
        self.storage.set(&key, value).map_err(Into::into)
    }

    pub fn set_layout(&mut self, layout: &Layout) -> Result<()> {
        self.set_with_namespace(constants::COMMON_NS, constants::LAYOUT, layout.to_toml()?)
    }

    pub fn layout(&self) -> Result<Layout> {
        let raw_layout =
            self.get_with_namespace::<String>(constants::COMMON_NS, constants::LAYOUT)?;
        Layout::parse(&raw_layout).map_err(Into::into)
    }

    pub fn set_root_key(&mut self, root_key: Ed25519PublicKey) -> Result<()> {
        let layout = self.layout()?;
        self.set_with_namespace(&layout.diem_root, DIEM_ROOT_KEY, root_key)
    }

    pub fn set_treasury_compliance_key(
        &mut self,
        treasury_compliance_key: Ed25519PublicKey,
    ) -> Result<()> {
        let layout = self.layout()?;
        self.set_with_namespace(
            &layout.diem_root,
            TREASURY_COMPLIANCE_KEY,
            treasury_compliance_key,
        )
    }

    pub fn set_operator_key(
        &mut self,
        operator_namespace: &str,
        operator_key: Ed25519PublicKey,
    ) -> Result<()> {
        todo!()
    }

    pub fn set_operator(
        &mut self,
        validator_namespace: &str,
        operator_namespace: &str,
    ) -> Result<()> {
        todo!()
    }

    pub fn set_owner_key(
        &mut self,
        owner_namespace: &str,
        owner_key: Ed25519PublicKey,
    ) -> Result<()> {
        todo!()
    }

    pub fn set_validator_config(
        &mut self,
        validator_namespace: &str,
        owner_name: &str,
        validator_address: &str,
        fullnode_address: &str,
        chain_id: ChainId,
    ) -> Result<()> {
        todo!()
    }
}
