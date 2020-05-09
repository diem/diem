script {
use 0x0::LibraBlock;
fun main(account: &signer) {
    LibraBlock::initialize_block_metadata(account);
}
}
// check: ABORTED
// check: 1
