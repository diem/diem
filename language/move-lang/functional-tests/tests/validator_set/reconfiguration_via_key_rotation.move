//! account: alice, 1000000, 0, validator
//! account: vivian, 1000000, 0, validator
//! account: viola, 1000000, 0, validator

//! new-transaction
//! sender: alice
script{
use 0x0::ValidatorConfig;
// rotate alice's pubkey
fun main() {
    ValidatorConfig::set_consensus_pubkey({{alice}}, x"40");
}
}

// check: events: []
// check: EXECUTED

//! block-prologue
//! proposer: vivian
//! block-time: 2

// not: NewEpochEvent
// check: EXECUTED

//! new-transaction
//! sender: vivian
script{
use 0x0::LibraSystem;
use 0x0::ValidatorConfig;

// rotate vivian's pubkey and then run the block prologue. Now, reconfiguration should be triggered.
fun main() {
    ValidatorConfig::set_consensus_pubkey({{vivian}}, x"40");
    LibraSystem::update_and_reconfigure();
    // check that the validator set contains Vivian's new key after reconfiguration
    0x0::Transaction::assert(*ValidatorConfig::get_consensus_pubkey(&LibraSystem::get_validator_config({{vivian}})) == x"40", 98);
}
}

// check: NewEpochEvent
// check: EXECUTED

//! block-prologue
//! proposer: vivian
//! block-time: 3

// check: EXECUTED

//! new-transaction
//! sender: vivian
script{
use 0x0::LibraSystem;
use 0x0::ValidatorConfig;
// rotate vivian's pubkey to the same value and run the block prologue. No reconfiguration should be
// triggered. the not "NewEpochEvent" check part tests this because reconfiguration always emits a
// NewEpoch event.
fun main() {
    ValidatorConfig::set_consensus_pubkey({{vivian}}, x"40");
    LibraSystem::update_and_reconfigure();
}
}

// not: NewEpochEvent
// check: EXECUTED
