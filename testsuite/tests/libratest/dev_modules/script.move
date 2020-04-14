// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use 0x0::LibraAccount;
use 0x0::LBR;
use {{sender}}::MyModule;

fun main(recipient: address, amount: u64) {
    let coin = LibraAccount::withdraw_from_sender<LBR::T>(amount);
    LibraAccount::deposit<LBR::T>(recipient, MyModule::id(coin));
}
