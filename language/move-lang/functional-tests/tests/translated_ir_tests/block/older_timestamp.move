//! account: vivian, 1000000, 0, validator

//! block-prologue
//! proposer: vivian
//! block-time: 100000000

//! new-transaction
script{
use 0x0::LibraBlock;
use 0x0::LibraTimestamp;

fun main() {
    0x0::Transaction::assert(LibraBlock::get_current_block_height() == 1, 76);
    0x0::Transaction::assert(LibraTimestamp::now_microseconds() == 100000000, 77);
}
}

//! block-prologue
//! proposer: vivian
//! block-time: 11000000

// check: ABORTED
// check: 5001
