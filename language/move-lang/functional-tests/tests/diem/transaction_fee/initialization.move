//! new-transaction
script {
use DiemFramework::TransactionFee;
fun main(account: signer) {
    let account = &account;
    TransactionFee::initialize(account);
}
}
// check: "Keep(ABORTED { code: 1,"
