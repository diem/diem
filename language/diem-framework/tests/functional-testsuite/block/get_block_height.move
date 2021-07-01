//! account: vivian, 1000000, 0, validator
//! new-transaction

script{
use 0x1::DiemBlock;
fun main() {
    // check that the height of the initial block is zero
    assert(DiemBlock::get_current_block_height() == 0, 77);
}
}

//! block-prologue
//! proposer: vivian
//! block-time: 100000000

//! new-transaction
script{
use 0x1::DiemBlock;
use 0x1::DiemTimestamp;

fun main() {
    assert(DiemBlock::get_current_block_height() == 1, 76);
    assert(DiemTimestamp::now_microseconds() == 100000000, 80);
}
}

//! block-prologue
//! proposer: vivian
//! block-time: 101000000

//! new-transaction
script{
use 0x1::DiemBlock;
use 0x1::DiemTimestamp;

fun main() {
    assert(DiemBlock::get_current_block_height() == 2, 76);
    assert(DiemTimestamp::now_microseconds() == 101000000, 80);
}
}
