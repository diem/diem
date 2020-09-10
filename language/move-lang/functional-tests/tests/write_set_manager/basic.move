//! new-transaction
script {
use 0x1::LibraWriteSetManager;
fun main(account: &signer) {
    LibraWriteSetManager::initialize(account);
}
}
// check: "Keep(ABORTED { code: 1,"
