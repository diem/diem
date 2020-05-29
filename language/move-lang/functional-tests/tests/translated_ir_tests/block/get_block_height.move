//! account: vivian, 1000000, 0, validator
//! new-transaction

script{
use 0x0::LibraBlock;
fun main() {
    // check that the height of the initial block is zero
    0x0::Transaction::assert(LibraBlock::get_current_block_height() == 0, 77);
}
}

//! block-prologue
//! proposer: vivian
//! block-time: 100000000

//! new-transaction
script{
use 0x0::LibraBlock;
use 0x0::LibraTimestamp;

fun main() {
    0x0::Transaction::assert(LibraBlock::get_current_block_height() == 1, 76);
    0x0::Transaction::assert(LibraTimestamp::now_microseconds() == 100000000, 80);
}
}

//! block-prologue
//! proposer: vivian
//! block-time: 101000000

//! new-transaction
script{
use 0x0::LibraBlock;
use 0x0::LibraTimestamp;

fun main() {
    0x0::Transaction::assert(LibraBlock::get_current_block_height() == 2, 76);
    0x0::Transaction::assert(LibraTimestamp::now_microseconds() == 101000000, 80);
}
}
