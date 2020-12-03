//! account: alice, 10000XUS
//! account: bob, 10000XUS

//! new-transaction
//! sender: alice
//! gas-price: 1
//! gas-currency: XUS
script {
fun main() {
}
}
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: bob
//! gas-price: 1
//! gas-currency: XUS
script {
fun main() {
    let x = 1;
    while (x < 2000) x = x + 1;
}
}
// check: "Keep(EXECUTED)"

//! new-transaction
script {
use 0x1::DiemAccount;
use 0x1::XUS::XUS;
fun main() {
    // Alice did less work than bob so she should pay less gas.
    assert(DiemAccount::balance<XUS>({{bob}}) < DiemAccount::balance<XUS>({{alice}}), 42);
}
}
// check: "Keep(EXECUTED)"
