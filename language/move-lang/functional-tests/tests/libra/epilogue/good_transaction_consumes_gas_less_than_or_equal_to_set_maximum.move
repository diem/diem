//! account: default, 10000Coin1

//! new-transaction
//! gas-price: 1
//! max-gas: 5000
//! gas-currency: Coin1
script {
fun main() {
}
}
// check: "Keep(EXECUTED)"

//! new-transaction
script {
use 0x1::LibraAccount;
use 0x1::Coin1::Coin1;
use 0x1::Signer;

fun main(account: &signer) {
    let sender = Signer::address_of(account);
    // Ensures that the account was deducted for the gas fee.
    assert(LibraAccount::balance<Coin1>(sender) < 10000, 42);
    // Ensures that we are not just charging max_gas for the transaction.
    assert(LibraAccount::balance<Coin1>(sender) >= 5000, 42);
}
}
// check: "Keep(EXECUTED)"
