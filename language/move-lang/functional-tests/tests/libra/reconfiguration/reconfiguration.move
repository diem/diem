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
    LibraConfig::reconfigure(account);
}
}
// check: NewEpochEvent
// check: event_data: "0200000000000000"
// check: "Keep(EXECUTED)"

//! block-prologue
//! proposer: vivian
//! block-time: 3

// Make sure two reconfigurations will only trigger one reconfiguration event.
//! new-transaction
//! sender: libraroot
script {
use 0x1::LibraConfig;

fun main(account: &signer) {
    LibraConfig::reconfigure(account);
}
}
// check: NewEpochEvent
// check: event_data: "0300000000000000"
// check: "Keep(EXECUTED)"
