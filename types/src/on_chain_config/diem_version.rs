// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    access_path::AccessPath,
    account_address::AccountAddress,
    account_config::CORE_CODE_ADDRESS,
    on_chain_config::{
        config_address, default_access_path_for_config, ConfigID, ConfigStorage, OnChainConfig,
    },
};
use move_core_types::{
    identifier::Identifier,
    language_storage::{StructTag, TypeTag},
};
use serde::{Deserialize, Serialize};

/// Defines the version of Diem Validator software.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct DiemVersion {
    pub major: u64,
}

impl OnChainConfig for DiemVersion {
    const IDENTIFIER: &'static str = "DiemVersion";

    fn fetch_config<T>(storage: &T) -> Option<Self>
    where
        T: ConfigStorage,
    {
        let access_path = match storage.fetch_config_access_path("DiemVersion".to_string()) {
            Some(path) => path,
            _ => default_access_path_for_config(Self::CONFIG_ID),
        };
        storage
            .fetch_config(access_path)
            .and_then(|bytes| Self::deserialize_into_config(&bytes).ok())
    }
}

pub fn experimental_version_access_path() -> AccessPath {
    AccessPath::new(
        config_address(),
        AccessPath::resource_access_vec(StructTag {
            address: CORE_CODE_ADDRESS,
            module: Identifier::new("CoreVersion").expect("failed to get identifier"),
            name: Identifier::new("CoreVersion").expect("failed to get identifier"),
            type_params: vec![TypeTag::Struct(StructTag {
                address: CORE_CODE_ADDRESS,
                module: Identifier::new("ExperimentalVersion").expect("failed to get identifier"),
                name: Identifier::new("ExperimentalVersion").expect("failed to get identifier"),
                type_params: vec![],
            })],
        }),
    )
}

// NOTE: version number for release 1.2 Diem
// Items gated by this version number include:
//  - the ScriptFunction payload type
pub const DIEM_VERSION_2: DiemVersion = DiemVersion { major: 2 };

// NOTE: version number for release 1.3 of Diem
// Items gated by this version number include:
//  - Multi-agent transactions
pub const DIEM_VERSION_3: DiemVersion = DiemVersion { major: 3 };

// NOTE: version number for release 1.4 of Diem
// Items gated by this version number include:
//  - Conflict-Resistant Sequence Numbers
pub const DIEM_VERSION_4: DiemVersion = DiemVersion { major: 4 };

// Maximum current known version
pub const DIEM_MAX_KNOWN_VERSION: DiemVersion = DIEM_VERSION_4;
