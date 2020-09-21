//! account: vivian, 1000000, 0, validator
//! account: alice, 1000000, 0

//! block-prologue
//! proposer: vivian
//! block-time: 1000000

//! new-transaction
script{
use 0x1::LibraBlock;
use 0x1::LibraTimestamp;

fun main() {
    assert(LibraBlock::get_current_block_height() == 1, 77);
    assert(LibraTimestamp::now_microseconds() == 1000000, 78);
}
}
// check: "Keep(EXECUTED)"

//! block-prologue
//! proposer: alice
//! block-time: 1000000

// check: UNEXPECTED_ERROR_FROM_KNOWN_MOVE_FUNCTION
