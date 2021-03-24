//! account: alice, 1000000XUS
//! account: bob, 1000000XUS
//! account: carol, 0XUS


// Transfer all of the Alice's funds to Carol. this script will execute successfully, but
// the epilogue will fail because Alice spent her gas deposit. The VM should revert the state
// changes and re-execute the epilogue. Alice will still be charged for the gas she used.

//! new-transaction
//! sender: alice
//! gas-price: 1
//! gas-currency: XUS
//! args: 1000000
script {
use 0x1::DiemAccount;
use 0x1::XUS::XUS;

fun main(account: signer, amount: u64) {
    let account = &account;
    let with_cap = DiemAccount::extract_withdraw_capability(account);
    DiemAccount::pay_from<XUS>(&with_cap, {{carol}}, amount, x"", x"");
    DiemAccount::restore_withdraw_capability(with_cap);
}
}
// check: "Keep(ABORTED { code: 257288,"


// Bob sends the same transaction script, with the amount set to 1000 instead of his full balance.
// This transaction should go through and bob will pay for the gas he used.

//! new-transaction
//! sender: bob
//! gas-price: 1
//! gas-currency: XUS
//! args: 1000
script {
use 0x1::DiemAccount;
use 0x1::XUS::XUS;

fun main(account: signer, amount: u64) {
    let account = &account;
    let with_cap = DiemAccount::extract_withdraw_capability(account);
    DiemAccount::pay_from<XUS>(&with_cap, {{carol}}, amount, x"", x"");
    DiemAccount::restore_withdraw_capability(with_cap);
}
}
// check: "Keep(EXECUTED)"


// Check that the following invariants holds:
// 1) Carol's balance has went up by 1000, receiving funds from Bob but not Alice.
// 2) Alice's balance is exactly 1000 greater than Bob's, indicating they consumed the same amount of gas.

//! new-transaction
script {
use 0x1::DiemAccount;
use 0x1::XUS::XUS;

fun main() {
    assert(DiemAccount::balance<XUS>({{carol}}) == 1000, 42);
    assert(DiemAccount::balance<XUS>({{alice}}) == DiemAccount::balance<XUS>({{bob}}) + 1000, 43)
}
}
// check: "Keep(EXECUTED)"
