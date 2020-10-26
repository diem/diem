//! account: default, 100000Coin1

//! new-transaction
//! gas-price: 1
//! gas-currency: Coin1
//! max-gas: 700
//! sender: default
script {
fun main() {
    loop {}
}
}
// check: "EXECUTION_FAILURE { status_code: OUT_OF_GAS,"
// check: "gas_used: 700,"
// check: "Keep(OUT_OF_GAS)"

//! new-transaction
//! sender: default
script {
use 0x1::LibraAccount;
use 0x1::Coin1::Coin1;
use 0x1::Signer;

fun main(account: &signer) {
    let sender = Signer::address_of(account);
    assert(LibraAccount::balance<Coin1>(sender) == 100000 - 700, 42);
}
}
// check: "Keep(EXECUTED)"
