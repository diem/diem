// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    access_path::{AccessPath, Accesses},
    account_config::{association_address, CORE_CODE_ADDRESS},
    language_storage::{StructTag, TypeTag},
    transaction::SCRIPT_HASH_LENGTH,
};
use libra_crypto::HashValue;
use move_core_types::identifier::Identifier;
use serde::{Deserialize, Serialize};

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
