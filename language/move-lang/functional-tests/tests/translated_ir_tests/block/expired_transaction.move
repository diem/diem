//! account: vivian, 1000000, 0, validator

//! block-prologue
//! proposer: vivian
//! block-time: 100000000

//! new-transaction
//! expiration-time: 100
script{
fun main() {
}
}
// check: TRANSACTION_EXPIRED

//! new-transaction
//! expiration-time: 101
script{
fun main() {
}
}
// check: EXECUTED

// TODO: 100 + 86400 = 86500, should be rejected after we fix the mempool flakiness. See details in issues #2346.
//! new-transaction
//! expiration-time: 86500
script{
fun main() {
}
}
// check: EXECUTED

//! block-prologue
//! proposer: vivian
//! block-time: 101000000

//! new-transaction
//! expiration-time: 86500
script{
fun main() {
}
}
// check: EXECUTED

//! new-transaction
//! expiration-time: 101
script{
fun main() {
}
}
// check: TRANSACTION_EXPIRED

//! new-transaction
//! expiration-time: 9223372036855
script{
fun main() {
}
}
// check: TRANSACTION_EXPIRED
