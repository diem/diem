// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use move_core_types::{
    account_address::AccountAddress,
    ident_str,
    identifier::IdentStr,
    move_resource::{MoveResource, MoveStructType},
};
#[cfg(any(test, feature = "fuzzing"))]
use proptest_derive::Arbitrary;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
pub struct ChainMarkerResource {
    pub config_address: AccountAddress,
}

impl MoveStructType for ChainMarkerResource {
    const MODULE_NAME: &'static IdentStr = ident_str!("ChainMarker");
    const STRUCT_NAME: &'static IdentStr = ident_str!("ChainMarker");
}

impl MoveResource for ChainMarkerResource {}
