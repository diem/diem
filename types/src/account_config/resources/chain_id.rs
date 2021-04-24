// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::chain_id::ChainId;
use move_core_types::{
    ident_str,
    identifier::IdentStr,
    move_resource::{MoveResource, MoveStructType},
};
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

impl MoveStructType for ChainIdResource {
    const MODULE_NAME: &'static IdentStr = ident_str!("ChainId");
    const STRUCT_NAME: &'static IdentStr = ident_str!("ChainId");
}

impl MoveResource for ChainIdResource {}
