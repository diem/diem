//! account: vivian, 1000000, 0, validator

//! block-prologue
//! proposer-address: 0x0
//! block-time: 1

// check: UNEXPECTED_ERROR_FROM_KNOWN_MOVE_FUNCTION

//! block-prologue
//! proposer-address: 0x0
//! block-time: 0

//! new-transaction
script {
    use 0x1::DiemTimestamp;
    fun main(account: &signer) {
        DiemTimestamp::set_time_has_started(account);
    }
}
// check: "Keep(ABORTED { code: 1,"

//! new-transaction
//! sender: diemroot
script {
    use 0x1::DiemTimestamp;
    fun main(account: &signer) {
        DiemTimestamp::set_time_has_started(account);
    }
}
// check: "Keep(ABORTED { code: 1,"
