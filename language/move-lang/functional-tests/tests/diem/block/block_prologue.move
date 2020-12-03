//! account: vivian, 1000000, 0, validator

//! block-prologue
//! proposer: vivian
//! block-time: 1000000

// check: EventKey([17, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 10, 85, 12, 24])
// check: NewBlockEvent
// check: 1000000

//! new-transaction
script{
use 0x1::DiemTimestamp;
use 0x1::DiemBlock;

fun main() {
    assert(DiemBlock::get_current_block_height() == 1, 73);
    assert(DiemTimestamp::now_microseconds() == 1000000, 76);
}
}
// check: "Keep(EXECUTED)"

//! new-transaction
script{
use 0x1::DiemTimestamp;

fun main() {
    assert(DiemTimestamp::now_microseconds() != 2000000, 77);
}
}
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: vivian
script{
use 0x1::DiemTimestamp;

fun main(account: &signer) {
    DiemTimestamp::update_global_time(account, {{vivian}}, 20);
}
}
// check: "Keep(ABORTED { code: 514,"
