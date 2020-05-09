//! account: vivian, 1000000, 0, validator

//! block-prologue
//! proposer: vivian
//! block-time: 3

//! new-transaction
script {
use 0x0::LibraTimestamp;
use 0x0::Transaction;

fun main() {
    Transaction::assert(!LibraTimestamp::is_genesis(), 10)
}
}
