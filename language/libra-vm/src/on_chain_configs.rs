// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_types::on_chain_config::{OnChainConfig, VMPublishingOption};
use move_vm_state::data_cache::RemoteCache;

#[derive(Debug, Clone)]
pub(crate) struct VMConfig {
    pub publishing_options: VMPublishingOption,
}

impl VMConfig {
    pub fn load_on_chain_config(state_view: &dyn RemoteCache) -> Option<Self> {
        if let Ok(publishing_options) = VMPublishingOption::fetch_config(&Box::new(state_view)) {
            Some(VMConfig { publishing_options })
        } else {
            None
        }
    }
}
