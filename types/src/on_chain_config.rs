// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    access_path::{AccessPath, Accesses},
    account_config::{association_address, CORE_CODE_ADDRESS},
    language_storage::{StructTag, TypeTag},
    transaction::SCRIPT_HASH_LENGTH,
};
use anyhow::{format_err, Result};
use libra_crypto::HashValue;
use move_core_types::identifier::Identifier;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};

/// To register an on-chain config in Rust:
/// 1. Implement the `OnChainConfig` trait for the Rust representation of the config
/// 2. Add the config's `ConfigID` to `ON_CHAIN_CONFIG_REGISTRY`

#[derive(Copy, Clone, Eq, Hash, PartialEq)]
pub struct ConfigID(&'static str);

impl ConfigID {
    pub fn access_path(self) -> AccessPath {
        access_path_for_config(Identifier::new(self.0).expect("failed to get Identifier"))
    }
}

/// State sync will panic if the value of any config in this registry is uninitialized
pub const ON_CHAIN_CONFIG_REGISTRY: &[ConfigID] = &[VMPublishingOption::CONFIG_ID];

#[derive(Clone)]
pub struct OnChainConfigPayload {
    configs: Arc<HashMap<ConfigID, Vec<u8>>>,
}

impl OnChainConfigPayload {
    pub fn new(configs: Arc<HashMap<ConfigID, Vec<u8>>>) -> Self {
        Self { configs }
    }

    pub fn get<T: OnChainConfig>(&self) -> Result<T> {
        let bytes = self
            .configs
            .get(&T::CONFIG_ID)
            .ok_or_else(|| format_err!("[on-chain cfg] config not in payload"))?;
        T::deserialize_into_config(bytes)
    }
}

/// Trait to be implemented by a storage type from which to read on-chain configs
pub trait ConfigStorage {
    fn fetch_config(&self, access_path: AccessPath) -> Option<Vec<u8>>;
}

/// Trait to be implemented by a Rust struct representation of an on-chain config
/// that is stored in storage as a deserialized byte array
pub trait OnChainConfig: Send + Sync + DeserializeOwned {
    const IDENTIFIER: &'static str;
    const CONFIG_ID: ConfigID = ConfigID(Self::IDENTIFIER);

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
}

pub fn access_path_for_config(config_name: Identifier) -> AccessPath {
    AccessPath::new(
        association_address(),
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

/// Defines and holds the publishing policies for the VM. There are three possible configurations:
/// 1. No module publishing, only whitelisted scripts are allowed.
/// 2. No module publishing, custom scripts are allowed.
/// 3. Both module publishing and custom scripts are allowed.
/// We represent these as an enum instead of a struct since whitelisting and module/script
/// publishing are mutually exclusive options.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum VMPublishingOption {
    /// Only allow scripts on a whitelist to be run
    Locked(Vec<[u8; SCRIPT_HASH_LENGTH]>),
    /// Allow custom scripts, but _not_ custom module publishing
    CustomScripts,
    /// Allow both custom scripts and custom module publishing
    Open,
}

impl OnChainConfig for VMPublishingOption {
    const IDENTIFIER: &'static str = "ScriptWhitelist";

    fn deserialize_into_config(bytes: &[u8]) -> Result<Self> {
        lcs::from_bytes::<Vec<u8>>(&bytes)
            .map_err(|e| {
                format_err!(
                    "Failed first round of deserialization for VMPublishingOption: {}",
                    e
                )
            })
            .and_then(|bytes| Self::deserialize_default_impl(&bytes))
    }
}

impl VMPublishingOption {
    pub fn is_open(&self) -> bool {
        match self {
            VMPublishingOption::Open => true,
            _ => false,
        }
    }

    pub fn is_allowed_script(&self, program: &[u8]) -> bool {
        match self {
            VMPublishingOption::Open | VMPublishingOption::CustomScripts => true,
            VMPublishingOption::Locked(whitelist) => {
                let hash_value = HashValue::from_sha3_256(program);
                whitelist.contains(hash_value.as_ref())
            }
        }
    }
}
