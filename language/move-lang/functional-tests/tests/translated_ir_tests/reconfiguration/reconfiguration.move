//! account: alice, 1000000, 0
//! account: vivian, 1000000, 0, validator
//! account: valentina, 1000000, 0, validator

//! block-prologue
//! proposer: vivian
//! block-time: 2

//! new-transaction
// Reconfiguration can only be invoked by association.
script {
use 0x0::LibraConfig;

fun main() {
    LibraConfig::reconfigure()
}
}

// check: ABORT
// check: 1

//! new-transaction
//! sender: config
script {
use 0x0::LibraConfig;

fun main() {
    LibraConfig::reconfigure()
}
}

// check: NewEpochEvent
// check: EXECUTED

//! new-transaction
//! sender: config
// Cannot trigger two reconfiguration within the same block.
script {
use 0x0::LibraConfig;

fun main() {
    LibraConfig::reconfigure()
}
}

// check: ABORT
// check: 23

//! block-prologue
//! proposer: vivian
//! block-time: 3

//! new-transaction
//! sender: config
script {
use 0x0::LibraConfig;

fun main() {
    LibraConfig::reconfigure()
}
}

// check: NewEpochEvent
// check: EXECUTED
