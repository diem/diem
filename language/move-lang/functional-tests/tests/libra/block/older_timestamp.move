//! account: vivian, 1000000, 0, validator

//! block-prologue
//! proposer: vivian
//! block-time: 100000000

//! new-transaction
script{
use 0x1::LibraBlock;
use 0x1::LibraTimestamp;

fun main() {
    assert(LibraBlock::get_current_block_height() == 1, 76);
    assert(LibraTimestamp::now_microseconds() == 100000000, 77);
}
}

//! block-prologue
//! proposer: vivian
//! block-time: 11000000

// check: UNEXPECTED_ERROR_FROM_KNOWN_MOVE_FUNCTION
