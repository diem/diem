//! new-transaction
//! sender: libraroot
script {
use 0x1::Libra;
fun main(account: &signer) {
    Libra::initialize(account);
}
}
// check: "Keep(ABORTED { code: 1,"
