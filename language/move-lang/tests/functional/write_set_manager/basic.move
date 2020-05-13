//! new-transaction
script {
use 0x0::LibraWriteSetManager;
fun main() {
    LibraWriteSetManager::initialize();
}
}
// check: ABORTED
// check: 1
