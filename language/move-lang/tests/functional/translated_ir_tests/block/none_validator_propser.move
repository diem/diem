//! account: vivian, 1000000, 0, validator
//! account: alice, 1000000, 0

//! block-prologue
//! proposer: vivian
//! block-time: 1000000

//! new-transaction
script{
use 0x0::LibraBlock;
use 0x0::LibraTimestamp;

fun main() {
    0x0::Transaction::assert(LibraBlock::get_current_block_height() == 1, 77);
    0x0::Transaction::assert(LibraTimestamp::now_microseconds() == 1000000, 78);
}
}
// check: EXECUTED

//! block-prologue
//! proposer: alice
//! block-time: 1000000

// check: ABORTED
// check: 5002
