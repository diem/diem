//! account: vivian, 1000000, 0, validator

//! block-prologue
//! proposer: vivian
//! block-time: 3

//! new-transaction
script {
use 0x1::LibraTimestamp;

fun main() {
    assert(!LibraTimestamp::is_genesis(), 10)
}
}
