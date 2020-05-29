//! account: vivian, 1000000, 0, validator

//! new-transaction
script {
    use 0x0::LibraTimestamp;
    fun main(account: &signer) {
        LibraTimestamp::initialize(account);
    }
}
// check: ABORTED
// check: 1

//! block-prologue
//! proposer-address: 0x0
//! block-time: 1

// check: ABORTED
// check: 5001

//! block-prologue
//! proposer-address: 0x0
//! block-time: 0
