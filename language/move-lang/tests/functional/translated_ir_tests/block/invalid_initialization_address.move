script {
use 0x0::LibraBlock;
fun main() {
    LibraBlock::initialize_block_metadata();
}
}
// check: ABORTED
// check: 1
