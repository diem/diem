//! account: alice, 10000Coin1
//! account: bob, 10000Coin1

//! new-transaction
//! sender: alice
//! gas-price: 1
//! gas-currency: Coin1
script {
fun main() {
}
}
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: bob
//! gas-price: 1
//! gas-currency: Coin1
script {
fun main() {
    let x = 1;
    while (x < 2000) x = x + 1;
}
}
// check: "Keep(EXECUTED)"

//! new-transaction
script {
use 0x1::LibraAccount;
use 0x1::Coin1::Coin1;
fun main() {
    // Alice did less work than bob so she should pay less gas.
    assert(LibraAccount::balance<Coin1>({{bob}}) < LibraAccount::balance<Coin1>({{alice}}), 42);
}
}
// check: "Keep(EXECUTED)"
