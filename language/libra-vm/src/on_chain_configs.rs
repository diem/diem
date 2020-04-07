// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_types::on_chain_config::{
    ConfigStorage, LibraVersion, OnChainConfig, VMPublishingOption,
};
use move_vm_state::data_cache::RemoteCache;

#[derive(Debug, Clone)]
pub struct VMConfig {
    pub publishing_options: VMPublishingOption,
    pub version: LibraVersion,
}

impl VMConfig {
    pub fn load_on_chain_config(state_view: &dyn RemoteCache) -> Option<Self> {
        let publishing_options = state_view
            .fetch_config(VMPublishingOption::CONFIG_ID.access_path())
            .and_then(|bytes| VMPublishingOption::deserialize_into_config(&bytes).ok())?;
        let version = state_view
            .fetch_config(LibraVersion::CONFIG_ID.access_path())
            .and_then(|bytes| LibraVersion::deserialize_into_config(&bytes).ok())?;
        Some(VMConfig {
            version,
            publishing_options,
        })
    }
}
