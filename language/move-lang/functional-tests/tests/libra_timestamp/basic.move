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
