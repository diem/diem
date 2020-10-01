//! account: alice, 1000000Coin1
//! account: bob, 1000000Coin1
//! account: carol, 0Coin1


// Transfer all of the Alice's funds to Carol. this script will execute successfully, but
// the epilogue will fail because Alice spent her gas deposit. The VM should revert the state
// changes and re-execute the epilogue. Alice will still be charged for the gas she used.

//! new-transaction
//! sender: alice
//! gas-price: 1
//! gas-currency: Coin1
//! args: 1000000
script {
use 0x1::LibraAccount;
use 0x1::Coin1::Coin1;

fun main(account: &signer, amount: u64) {
    let with_cap = LibraAccount::extract_withdraw_capability(account);
    LibraAccount::pay_from<Coin1>(&with_cap, {{carol}}, amount, x"", x"");
    LibraAccount::restore_withdraw_capability(with_cap);
}
}
// check: "Keep(ABORTED { code: 257288,"


// Bob sends the same transaction script, with the amount set to 1000 instead of his full balance.
// This transaction should go through and bob will pay for the gas he used.

//! new-transaction
//! sender: bob
//! gas-price: 1
//! gas-currency: Coin1
//! args: 1000
script {
use 0x1::LibraAccount;
use 0x1::Coin1::Coin1;

fun main(account: &signer, amount: u64) {
    let with_cap = LibraAccount::extract_withdraw_capability(account);
    LibraAccount::pay_from<Coin1>(&with_cap, {{carol}}, amount, x"", x"");
    LibraAccount::restore_withdraw_capability(with_cap);
}
}
// check: "Keep(EXECUTED)"


// Check that the following invariants holds:
// 1) Carol's balance has went up by 1000, receiving funds from Bob but not Alice.
// 2) Alice's balance is exactly 1000 greater than Bob's, indicating they consumed the same amount of gas.

//! new-transaction
script {
use 0x1::LibraAccount;
use 0x1::Coin1::Coin1;

fun main() {
    assert(LibraAccount::balance<Coin1>({{carol}}) == 1000, 42);
    assert(LibraAccount::balance<Coin1>({{alice}}) == LibraAccount::balance<Coin1>({{bob}}) + 1000, 43)
}
}
// check: "Keep(EXECUTED)"
