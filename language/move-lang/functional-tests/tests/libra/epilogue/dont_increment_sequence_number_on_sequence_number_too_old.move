// Set initial sequence number to 10.
//! account: default, 1000000, 10

//! sequence-number: 9
script {
fun main() {
}
}
// check: SEQUENCE_NUMBER_TOO_OLD

//! new-transaction
//! sequence-number: 10
script {
fun main() {
}
}
// check: "Keep(EXECUTED)"
