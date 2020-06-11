//! new-transaction
script {
use 0x1::LibraWriteSetManager;
fun main(account: &signer) {
    LibraWriteSetManager::initialize(account);
}
}
// check: ABORTED
// check: 1
