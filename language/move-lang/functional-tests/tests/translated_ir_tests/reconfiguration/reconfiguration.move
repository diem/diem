//! account: alice, 1000000, 0
//! account: vivian, 1000000, 0, validator
//! account: valentina, 1000000, 0, validator

//! block-prologue
//! proposer: vivian
//! block-time: 2

//! new-transaction
// Reconfiguration can only be invoked by the libra root.
script {
use 0x1::LibraConfig;

fun main(account: &signer) {
    LibraConfig::reconfigure(account);
}
}

// check: "Keep(ABORTED { code: 2,"

//! new-transaction
//! sender: libraroot
script {
use 0x1::LibraConfig;

fun main(account: &signer) {
    LibraConfig::reconfigure(account);
}
}
// check: NewEpochEvent
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: libraroot
// Cannot trigger two reconfiguration within the same block.
script {
use 0x1::LibraConfig;

fun main(account: &signer) {
    LibraConfig::reconfigure(account);
}
}
// check: "Keep(ABORTED { code: 1025,"
