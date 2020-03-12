// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_types::on_chain_config::{access_path_for_config, VMPublishingOption};
use move_core_types::identifier::Identifier;
use move_vm_state::data_cache::RemoteCache;
use once_cell::sync::Lazy;

#[derive(Debug, Clone)]
pub(crate) struct VMConfig {
    pub publishing_options: VMPublishingOption,
}

impl VMConfig {
    pub fn load_on_chain_config(state_view: &dyn RemoteCache) -> Option<Self> {
        Some(VMConfig {
            publishing_options: load_script_whitelist(state_view)?,
        })
    }
}

static SCRIPT_WHITELIST: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("ScriptWhitelist").unwrap());

fn load_script_whitelist(state_view: &dyn RemoteCache) -> Option<VMPublishingOption> {
    let access_path = access_path_for_config(SCRIPT_WHITELIST.clone());
    state_view
        .get(&access_path)
        .ok()?
        .and_then(|bytes| lcs::from_bytes::<Vec<u8>>(&bytes).ok())
        .and_then(|bytes| lcs::from_bytes::<VMPublishingOption>(&bytes).ok())
}
