//! new-transaction
script {
use 0x1::TransactionFee;
fun main(account: signer) {
    let account = &account;
    TransactionFee::initialize(account);
}
}
// check: "Keep(ABORTED { code: 1,"
