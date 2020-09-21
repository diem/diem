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
// check: "Keep(EXECUTED)"

//! new-transaction
//! expiration-time: 86500
script{
fun main() {
}
}
// check: "Keep(EXECUTED)"

//! block-prologue
//! proposer: vivian
//! block-time: 101000000

//! new-transaction
//! expiration-time: 86500
script{
fun main() {
}
}
// check: "Keep(EXECUTED)"

//! new-transaction
//! expiration-time: 101
script{
fun main() {
}
}
// check: TRANSACTION_EXPIRED
