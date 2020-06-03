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

// check: EXECUTED

//! new-transaction
//! sender: alice
script {
use 0x0::LibraSystem;
fun main(account: &signer) {
    LibraSystem::remove_validator(account, {{alice}});
}
}

// check: NewEpochEvent
// check: EXECUTED

//! new-transaction
//! sender: bob
script {
use 0x0::LibraSystem;
fun main(account: &signer) {
    LibraSystem::remove_validator(account, {{bob}});
}
}

// check: ABORTED

//! block-prologue
//! proposer: bob
//! block-time: 3

// check: EXECUTED

//! new-transaction
//! sender: bob
script {
use 0x0::LibraSystem;
fun main(account: &signer) {
    LibraSystem::remove_validator(account, {{bob}});
}
}

// check: NewEpochEvent
// check: EXECUTED
