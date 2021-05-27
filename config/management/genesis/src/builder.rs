// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::layout::Layout;
use anyhow::Result;
use diem_crypto::ed25519::{Ed25519PrivateKey, Ed25519PublicKey};
use diem_global_constants::{
    CONSENSUS_KEY, DIEM_ROOT_KEY, EXECUTION_KEY, FULLNODE_NETWORK_KEY, OPERATOR_KEY, OWNER_KEY,
    SAFETY_DATA, TREASURY_COMPLIANCE_KEY, VALIDATOR_NETWORK_KEY, WAYPOINT,
};
use diem_management::{
    config::ConfigPath,
    constants::{self, VALIDATOR_OPERATOR},
    error::Error,
    secure_backend::SharedBackend,
};
use diem_secure_storage::{CryptoStorage, KVStorage, Namespaced, Storage};
use diem_types::chain_id::ChainId;
use serde::{de::DeserializeOwned, Serialize};

pub struct GenesisBuilder<S> {
    storage: S,
}

impl<S> GenesisBuilder<S> {
    pub fn new(storage: S) -> Self {
        Self { storage }
    }
}

impl<S: KVStorage> GenesisBuilder<S> {
    fn with_namespace(&self, namespace: &str) -> Namespaced<&S> {
        Namespaced::new(namespace, &self.storage)
    }

    fn with_namespace_mut(&mut self, namespace: &str) -> Namespaced<&mut S> {
        Namespaced::new(namespace, &mut self.storage)
    }

    pub fn set_layout(&mut self, layout: &Layout) -> Result<()> {
        self.with_namespace_mut(constants::COMMON_NS)
            .set(constants::LAYOUT, layout.to_toml()?)
            .map_err(Into::into)
    }

    pub fn layout(&self) -> Result<Layout> {
        let raw_layout = self
            .with_namespace(constants::COMMON_NS)
            .get::<String>(constants::LAYOUT)?
            .value;
        Layout::parse(&raw_layout).map_err(Into::into)
    }

    pub fn set_root_key(&mut self, root_key: Ed25519PublicKey) -> Result<()> {
        let layout = self.layout()?;
        self.with_namespace_mut(&layout.diem_root)
            .set(DIEM_ROOT_KEY, root_key)
            .map_err(Into::into)
    }

    pub fn set_treasury_compliance_key(
        &mut self,
        treasury_compliance_key: Ed25519PublicKey,
    ) -> Result<()> {
        let layout = self.layout()?;
        self.with_namespace_mut(&layout.diem_root)
            .set(TREASURY_COMPLIANCE_KEY, treasury_compliance_key)
            .map_err(Into::into)
    }

    pub fn set_operator_key(
        &mut self,
        operator_namespace: &str,
        operator_key: Ed25519PublicKey,
    ) -> Result<()> {
        self.with_namespace_mut(&operator_namespace)
            .set(OPERATOR_KEY, operator_key)
            .map_err(Into::into)
    }

    pub fn set_operator(&mut self, validator: &str, operator: &str) -> Result<()> {
        self.with_namespace_mut(validator)
            .set(VALIDATOR_OPERATOR, operator)
            .map_err(Into::into)
    }

    pub fn set_owner_key(
        &mut self,
        owner_namespace: &str,
        owner_key: Ed25519PublicKey,
    ) -> Result<()> {
        self.with_namespace_mut(&owner_namespace)
            .set(OWNER_KEY, owner_key)
            .map_err(Into::into)
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
