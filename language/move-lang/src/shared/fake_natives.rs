// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use crate::parser::ast::ModuleIdent;

/// 'Native' functions that are actually bytecode isntructions

//**************************************************************************************************
// Transaction
//**************************************************************************************************

/// Fake module around transaction meta data
pub mod transaction {
    pub const MOD: &str = "Transaction";

    pub const SENDER: &str = "sender";
}

pub fn is_fake_native(mident: &ModuleIdent) -> bool {
    mident.0.value.address == Address::LIBRA_CORE && mident.0.value.name.value() == transaction::MOD
}
