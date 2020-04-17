// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    access_path::{AccessPath, Accesses},
    account_address::AccountAddress,
    account_config::{association_address, CORE_CODE_ADDRESS},
    event::{EventHandle, EventKey},
    language_storage::{StructTag, TypeTag},
};
use anyhow::{format_err, Result};
use move_core_types::identifier::Identifier;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};

mod libra_version;
mod validator_set;
mod vm_config;

pub use self::{
    libra_version::LibraVersion,
    validator_set::ValidatorSet,
    vm_config::{VMConfig, VMPublishingOption},
};
use crate::move_resource::MoveResource;

/// To register an on-chain config in Rust:
/// 1. Implement the `OnChainConfig` trait for the Rust representation of the config
/// 2. Add the config's `ConfigID` to `ON_CHAIN_CONFIG_REGISTRY`

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub struct ConfigID(&'static str, &'static str);

impl ConfigID {
    pub fn access_path(self) -> AccessPath {
        access_path_for_config(
            AccountAddress::from_hex_literal(self.0).expect("failed to get address"),
            Identifier::new(self.1).expect("failed to get Identifier"),
        )
    }
}

/// State sync will panic if the value of any config in this registry is uninitialized
pub const ON_CHAIN_CONFIG_REGISTRY: &[ConfigID] = &[
    VMConfig::CONFIG_ID,
    LibraVersion::CONFIG_ID,
    ValidatorSet::CONFIG_ID,
];

#[derive(Clone, Debug, PartialEq)]
pub struct OnChainConfigPayload {
    epoch: u64,
    configs: Arc<HashMap<ConfigID, Vec<u8>>>,
}

impl OnChainConfigPayload {
    pub fn new(epoch: u64, configs: Arc<HashMap<ConfigID, Vec<u8>>>) -> Self {
        Self { epoch, configs }
    }

    pub fn epoch(&self) -> u64 {
        self.epoch
    }

    pub fn get<T: OnChainConfig>(&self) -> Result<T> {
        let bytes = self
            .configs
            .get(&T::CONFIG_ID)
            .ok_or_else(|| format_err!("[on-chain cfg] config not in payload"))?;
        T::deserialize_into_config(bytes)
    }

    pub fn configs(&self) -> &HashMap<ConfigID, Vec<u8>> {
        &self.configs
    }
}

/// Trait to be implemented by a storage type from which to read on-chain configs
pub trait ConfigStorage {
    fn fetch_config(&self, access_path: AccessPath) -> Option<Vec<u8>>;
}

/// Trait to be implemented by a Rust struct representation of an on-chain config
/// that is stored in storage as a serialized byte array
pub trait OnChainConfig: Send + Sync + DeserializeOwned {
    // association_address
    const ADDRESS: &'static str = "0xA550C18";
    const IDENTIFIER: &'static str;
    const CONFIG_ID: ConfigID = ConfigID(Self::ADDRESS, Self::IDENTIFIER);

    // Single-round LCS deserialization from bytes to `Self`
    // This is the expected deserialization pattern for most Rust representations,
    // but sometimes `deserialize_into_config` may need an extra customized round of deserialization
    // (e.g. enums like `VMPublishingOption`)
    // In the override, we can reuse this default logic via this function
    // Note: we cannot directly call the default `deserialize_into_config` implementation
    // in its override - this will just refer to the override implementation itself
    fn deserialize_default_impl(bytes: &[u8]) -> Result<Self> {
        lcs::from_bytes::<Self>(&bytes)
            .map_err(|e| format_err!("[on-chain config] Failed to deserialize into config: {}", e))
    }

    // Function for deserializing bytes to `Self`
    // It will by default try one round of LCS deserialization directly to `Self`
    // The implementation for the concrete type should override this function if this
    // logic needs to be customized
    fn deserialize_into_config(bytes: &[u8]) -> Result<Self> {
        Self::deserialize_default_impl(bytes)
    }

    fn fetch_config<T>(storage: T) -> Option<Self>
    where
        T: ConfigStorage,
    {
        storage
            .fetch_config(Self::CONFIG_ID.access_path())
            .and_then(|bytes| Self::deserialize_into_config(&bytes).ok())
    }
}

pub fn new_epoch_event_key() -> EventKey {
    EventKey::new_from_address(&association_address(), 4)
}

pub fn access_path_for_config(address: AccountAddress, config_name: Identifier) -> AccessPath {
    AccessPath::new(
        address,
        AccessPath::resource_access_vec(
            &StructTag {
                address: CORE_CODE_ADDRESS,
                module: Identifier::new("LibraConfig").unwrap(),
                name: Identifier::new("T").unwrap(),
                type_params: vec![TypeTag::Struct(StructTag {
                    address: CORE_CODE_ADDRESS,
                    module: config_name,
                    name: Identifier::new("T").unwrap(),
                    type_params: vec![],
                })],
            },
            &Accesses::empty(),
        ),
    )
}

#[derive(Deserialize, Serialize)]
pub struct ConfigurationResource {
    epoch: u64,
    last_reconfiguration_time: u64,
    events: EventHandle,
}

impl ConfigurationResource {
    pub fn epoch(&self) -> u64 {
        self.epoch
    }
    pub fn last_reconfiguration_time(&self) -> u64 {
        self.last_reconfiguration_time
    }
}

impl MoveResource for ConfigurationResource {
    const MODULE_NAME: &'static str = "LibraConfig";
    const STRUCT_NAME: &'static str = "Configuration";
}
