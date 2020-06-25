// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

// Note: If this test file fails to run, it is possible that the
// compiled version of the Move stdlib needs to be updated. This code
// is compiled with the latest compiler and stdlib, but it runs with
// the compiled stdlib.

script {
use 0x1::LibraAccount;
use 0x1::Coin1::Coin1;
use {{sender}}::MyModule;

fun main(account: &signer, recipient: address, amount: u64) {
    let with_cap = LibraAccount::extract_withdraw_capability(account);
    let coin = LibraAccount::withdraw_from<Coin1>(&with_cap, amount);
    LibraAccount::restore_withdraw_capability(with_cap);
    LibraAccount::deposit<Coin1>(account, recipient, MyModule::id(coin));
}
}
