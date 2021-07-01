script {
use 0x1::DiemBlock;
fun main(account: signer) {
    DiemBlock::initialize_block_metadata(&account);
}
}
