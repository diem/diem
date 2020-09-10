script {
use 0x1::LibraBlock;
fun main(account: &signer) {
    LibraBlock::initialize_block_metadata(account);
}
}
// check: "Keep(ABORTED { code: 1,"
