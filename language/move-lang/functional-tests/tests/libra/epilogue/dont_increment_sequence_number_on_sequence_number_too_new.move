//! account: default, 1000000, 0

//! sequence-number: 5
script {
fun main() {
}
}
// check: SEQUENCE_NUMBER_TOO_NEW

// Running with 0 should succeed because sequence number wasn't bumped.
//! new-transaction
//! sequence-number: 0
script {
fun main() {
}
}
// check: "Keep(EXECUTED)"
