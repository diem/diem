// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account_address::AccountAddress, account_config::constants::ACCOUNT_MODULE_IDENTIFIER,
};
use move_core_types::{
    ident_str,
    identifier::IdentStr,
    move_resource::{MoveResource, MoveStructType},
};
#[cfg(any(test, feature = "fuzzing"))]
use proptest_derive::Arbitrary;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
pub struct WithdrawCapabilityResource {
    account_address: AccountAddress,
}

impl MoveStructType for WithdrawCapabilityResource {
    const MODULE_NAME: &'static IdentStr = ACCOUNT_MODULE_IDENTIFIER;
    const STRUCT_NAME: &'static IdentStr = ident_str!("WithdrawCapability");
}

impl MoveResource for WithdrawCapabilityResource {}
