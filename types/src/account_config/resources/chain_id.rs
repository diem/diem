// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::chain_id::ChainId;
use move_core_types::move_resource::MoveResource;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ChainIdResource {
    chain_id: u8,
}

impl ChainIdResource {
    pub fn chain_id(&self) -> ChainId {
        ChainId::new(self.chain_id)
    }
}

impl MoveResource for ChainIdResource {
    const MODULE_NAME: &'static str = "ChainId";
    const STRUCT_NAME: &'static str = "ChainId";
}
