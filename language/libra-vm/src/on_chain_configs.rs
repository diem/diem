// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_logger::prelude::*;
use libra_types::on_chain_config::{ConfigStorage, OnChainConfig, VMPublishingOption};
use move_vm_state::data_cache::RemoteCache;

#[derive(Debug, Clone)]
pub(crate) struct VMConfig {
    pub publishing_options: VMPublishingOption,
}

impl VMConfig {
    pub fn load_on_chain_config(state_view: &dyn RemoteCache) -> Option<Self> {
        if let Some(bytes) = state_view.fetch_config(VMPublishingOption::CONFIG_ID.access_path()) {
            match VMPublishingOption::deserialize_into_config(&bytes) {
                Ok(publishing_options) => return Some(VMConfig { publishing_options }),
                Err(e) => error!("[VM config] failed to deserialize into config: {:?}", e),
            }
        }
        None
    }
}
