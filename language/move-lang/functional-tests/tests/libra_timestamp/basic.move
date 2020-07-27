//! account: vivian, 1000000, 0, validator

//! new-transaction
script {
    use 0x1::LibraTimestamp;
    fun main(account: &signer) {
        LibraTimestamp::initialize(account);
    }
}
// TODO(status_migration) remove duplicate check
// check: ABORTED
// check: ABORTED
// check: 1

//! block-prologue
//! proposer-address: 0x0
//! block-time: 1

// check: ABORTED
// check: 2

//! block-prologue
//! proposer-address: 0x0
//! block-time: 0

//! new-transaction
script {
    use 0x1::LibraTimestamp;
    fun main(account: &signer) {
        LibraTimestamp::set_time_has_started(account);
    }
}
// check: ABORTED
// check: "code: 0"

//! new-transaction
//! sender: libraroot
script {
    use 0x1::LibraTimestamp;
    fun main(account: &signer) {
        LibraTimestamp::set_time_has_started(account);
    }
}
// check: ABORTED
// check: "code: 4"
