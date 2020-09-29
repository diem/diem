//! account: vivian, 1000000, 0, validator

//! block-prologue
//! proposer: vivian
//! block-time: 1000000

// check: EventKey([17, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 10, 85, 12, 24])
// check: NewBlockEvent
// check: 1000000

//! new-transaction
script{
use 0x1::LibraTimestamp;
use 0x1::LibraBlock;

fun main() {
    assert(LibraBlock::get_current_block_height() == 1, 73);
    assert(LibraTimestamp::now_microseconds() == 1000000, 76);
}
}

//! new-transaction
script{
use 0x1::LibraTimestamp;

fun main() {
    assert(LibraTimestamp::now_microseconds() != 2000000, 77);
}
}

//! new-transaction
//! sender: vivian
script{
use 0x1::LibraTimestamp;

fun main(account: &signer) {
    LibraTimestamp::update_global_time(account, {{vivian}}, 20);
}
}
// check: "Keep(ABORTED { code: 514,"
