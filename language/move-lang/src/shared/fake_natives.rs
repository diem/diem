// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

/// 'Native' functions that are actually bytecode isntructions

//**************************************************************************************************
// Transaction
//**************************************************************************************************

/// Fake module around transaction meta data
pub mod transaction {
    pub const MOD: &str = "Transaction";

    pub const GAS_PRICE: &str = "gas_unit_price";
    pub const MAX_GAS: &str = "max_gas_units";
    pub const GAS_REMAINING: &str = "gas_remaining";
    pub const SENDER: &str = "sender";
    pub const SEQUENCE_NUM: &str = "sequence_number";
    pub const PUBLIC_KEY: &str = "public_key";
    /// 'Inlined' during hlir::translate
    pub const ASSERT: &str = "assert";
}
