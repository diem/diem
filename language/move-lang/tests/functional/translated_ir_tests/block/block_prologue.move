//! account: vivian, 1000000, 0, validator

//! block-prologue
//! proposer: vivian
//! block-time: 1000000

// check: EventKey([16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 10, 85, 12, 24])
// check: NewBlockEvent
// check: 1000000

//! new-transaction
script{
use 0x0::LibraTimestamp;
use 0x0::LibraBlock;

fun main() {
    0x0::Transaction::assert(LibraBlock::get_current_block_height() == 1, 73);
    0x0::Transaction::assert(LibraTimestamp::now_microseconds() == 1000000, 76);
}
}

//! new-transaction
script{
use 0x0::LibraTimestamp;

fun main() {
    0x0::Transaction::assert(LibraTimestamp::now_microseconds() != 2000000, 77);
}
}
//! new-transaction
//! sender: vivian
script{
use 0x0::LibraBlock;
use 0x0::Vector;

fun main() {
    LibraBlock::block_prologue(1, 10, Vector::empty<address>(), {{vivian}});
}
}
// check: ABORTED
// check: 33

//! new-transaction
//! sender: vivian
script{
use 0x0::LibraTimestamp;

fun main() {
    LibraTimestamp::update_global_time({{vivian}}, 20);
}
}
// check: ABORTED
// check: 33
