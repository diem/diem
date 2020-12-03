//! account: default, 100000XUS

//! new-transaction
//! gas-price: 1
//! gas-currency: XUS
//! max-gas: 700
//! sender: default
script {
fun main() {
    while(true) {}
}
}
// check: "EXECUTION_FAILURE { status_code: OUT_OF_GAS,"
// check: "gas_used: 700,"
// check: "Keep(OUT_OF_GAS)"

//! new-transaction
//! sender: default
script {
use 0x1::DiemAccount;
use 0x1::XUS::XUS;
use 0x1::Signer;

fun main(account: &signer) {
    let sender = Signer::address_of(account);
    assert(DiemAccount::balance<XUS>(sender) == 100000 - 700, 42);
}
}
// check: "Keep(EXECUTED)"
