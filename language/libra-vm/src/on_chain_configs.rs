// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_logger::prelude::*;
use libra_types::on_chain_config::{LibraVersion, OnChainConfig, VMPublishingOption};
use move_vm_state::data_cache::RemoteCache;

#[derive(Debug, Clone)]
pub struct VMConfig {
    pub publishing_options: VMPublishingOption,
    pub version: LibraVersion,
}

impl VMConfig {
    pub fn load_on_chain_config(state_view: &dyn RemoteCache) -> Option<Self> {
        let publishing_options = VMPublishingOption::fetch_config(state_view);
        let version = LibraVersion::fetch_config(state_view);
        if publishing_options.is_some() && version.is_some() {
            Some(VMConfig {
                publishing_options: publishing_options?,
                version: version?,
            })
        } else {
            error!("Failed to fetch VM config");
            None
        }
    }
}
