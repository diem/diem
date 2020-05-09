//! new-transaction
script {
use 0x0::LibraWriteSetManager;
fun main(account: &signer) {
    LibraWriteSetManager::initialize(account);
}
}
// check: ABORTED
// check: 1
