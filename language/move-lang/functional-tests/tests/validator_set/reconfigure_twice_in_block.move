// Checks that only one change to the validator set can be made per block:
// if the first transaction removes a validator and the second transaction
// removes a validator in the same block, then the first transaction will succeed
// and the second will fail.

//! account: alice, 1000000, 0, validator
//! account: bob, 1000000, 0, validator
//! account: carrol, 1000000, 0, validator

//! block-prologue
//! proposer: bob
//! block-time: 2

// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: libraroot
script {
    use 0x1::LibraSystem;
    fun main(account: &signer) {
        LibraSystem::remove_validator(account, {{alice}});
    }
}

// check: NewEpochEvent
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: libraroot
script {
    use 0x1::LibraSystem;
    fun main(account: &signer) {
        LibraSystem::remove_validator(account, {{bob}});
    }
}

// check: "Keep(ABORTED { code: 1025,"

//! block-prologue
//! proposer: bob
//! block-time: 3

// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: libraroot
script {
    use 0x1::LibraSystem;
    fun main(account: &signer) {
        LibraSystem::remove_validator(account, {{bob}});
    }
}

// check: NewEpochEvent
// check: "Keep(EXECUTED)"
