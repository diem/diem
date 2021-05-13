// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

// Note: If this test file fails to run, it is possible that the
// compiled version of the Move stdlib needs to be updated. This code
// is compiled with the latest compiler and stdlib, but it runs with
// the compiled stdlib.

script {
use DiemFramework::Diem;
use DiemFramework::DiemAccount;
use DiemFramework::XUS::XUS;
use {{sender}}::MyModule;

fun main(account: signer, recipient: address, amount: u64) {
    let with_cap = DiemAccount::extract_withdraw_capability(&account);
    DiemAccount::pay_from<XUS>(&with_cap, recipient, amount, x"", x"");
    DiemAccount::restore_withdraw_capability(with_cap);
    let coin = MyModule::id<XUS>(Diem::zero<XUS>());
    Diem::destroy_zero(coin)
}
}
