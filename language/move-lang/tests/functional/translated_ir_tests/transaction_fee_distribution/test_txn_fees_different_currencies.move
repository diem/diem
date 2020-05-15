//! account: bob, 10000LBR
//! account: alice, 10000Coin1
//! account: gary, 10000Coin2

//! new-transaction
//! sender: bob
//! gas-price: 1
script {
fun main() { }
}
//! check: EXECUTED

//! new-transaction
//! sender: alice
//! gas-price: 1
//! gas-currency: Coin1
script {
fun main() { }
}
//! check: EXECUTED

//! new-transaction
//! sender: gary
//! gas-price: 1
//! gas-currency: Coin2
script {
fun main() { }
}
//! check: EXECUTED
