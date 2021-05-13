//! account: default, 10000XUS

//! new-transaction
//! gas-price: 1
//! max-gas: 5000
//! gas-currency: XUS
script {
fun main() {
}
}
// check: "Keep(EXECUTED)"

//! new-transaction
script {
use DiemFramework::DiemAccount;
use DiemFramework::XUS::XUS;
use Std::Signer;

fun main(account: signer) {
    let account = &account;
    let sender = Signer::address_of(account);
    // Ensures that the account was deducted for the gas fee.
    assert(DiemAccount::balance<XUS>(sender) < 10000, 42);
    // Ensures that we are not just charging max_gas for the transaction.
    assert(DiemAccount::balance<XUS>(sender) >= 5000, 42);
}
}
// check: "Keep(EXECUTED)"
