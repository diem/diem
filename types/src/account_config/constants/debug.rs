// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{account_config::constants::CORE_CODE_ADDRESS, language_storage::ModuleId};
use move_core_types::identifier::Identifier;
use once_cell::sync::Lazy;

// Libra
static GENESIS_MODULE_NAME: Lazy<Identifier> = Lazy::new(|| Identifier::new("Genesis").unwrap());
pub static GENESIS_MODULE: Lazy<ModuleId> =
    Lazy::new(|| ModuleId::new(CORE_CODE_ADDRESS, GENESIS_MODULE_NAME.clone()));

// Debug
pub static DEBUG_MODULE_NAME: Lazy<Identifier> = Lazy::new(|| Identifier::new("Debug").unwrap());

pub static DEBUG_MODULE: Lazy<ModuleId> =
    Lazy::new(|| ModuleId::new(CORE_CODE_ADDRESS, DEBUG_MODULE_NAME.clone()));
