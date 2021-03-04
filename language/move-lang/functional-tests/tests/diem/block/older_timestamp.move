//! account: vivian, 1000000, 0, validator

//! block-prologue
//! proposer: vivian
//! block-time: 100000000

//! new-transaction
script{
use 0x1::DiemBlock;
use 0x1::DiemTimestamp;

fun main() {
    assert(DiemBlock::get_current_block_height() == 1, 76);
    assert(DiemTimestamp::now_microseconds() == 100000000, 77);
}
}

//! block-prologue
//! proposer: vivian
//! block-time: 11000000
