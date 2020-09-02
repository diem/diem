//! new-transaction
script {
use 0x1::LibraTransaction;
fun main(account: &signer) {
    LibraTransaction::initialize(account);
}
}
// check: ABORTED
// check: 1
