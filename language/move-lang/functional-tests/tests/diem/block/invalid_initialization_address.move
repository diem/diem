script {
use DiemFramework::DiemBlock;
fun main(account: signer) {
    DiemBlock::initialize_block_metadata(&account);
}
}
