// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    access_path::{AccessPath, Accesses},
    account_config::{association_address, CORE_CODE_ADDRESS},
    language_storage::{StructTag, TypeTag},
    transaction::SCRIPT_HASH_LENGTH,
};
use anyhow::Result;
use libra_crypto::HashValue;
use move_core_types::identifier::Identifier;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

/// Trait to be implemented by a storage type from which to read on-chain configs
pub trait ConfigStorage {
    fn fetch_config(&self, access_path: AccessPath) -> Option<Vec<u8>>;
}

/// Trait to be implemented by a Rust struct representation of an on-chain config
/// that is stored in storage as a deserialized byte array
pub trait OnChainConfig {
    const IDENTIFIER: &'static str;

    fn deserialize_into_config(bytes: &[u8]) -> Result<Self>
    where
        Self: DeserializeOwned,
    {
        let result = lcs::from_bytes::<Self>(&bytes).map_err(|e| {
            anyhow::format_err!(
                "Failed to serialize account blob bytes to on-chain config: {}",
                e
            )
        })?;
        Ok(result)
    }

    fn fetch_config<U: ConfigStorage>(storage: &U) -> Result<Self>
    where
        Self: DeserializeOwned,
    {
        let access_path = access_path_for_config(Identifier::new(Self::IDENTIFIER).unwrap());
        let bytes = storage
            .fetch_config(access_path)
            .ok_or_else(|| anyhow::format_err!("no config found"))?;
        Self::deserialize_into_config(&bytes)
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
