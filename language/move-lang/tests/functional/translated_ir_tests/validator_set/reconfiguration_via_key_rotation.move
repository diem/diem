//! account: alice
//! account: vivian, 1000000, 0, validator
//! account: viola, 1000000, 0, validator

//! new-transaction
//! sender: alice
script{
use 0x0::ValidatorConfig;

// register Alice as a validator candidate
fun main() {
    ValidatorConfig::register_candidate_validator(x"10", x"20", x"30", x"40", x"50", x"60");
}
}

// check: EXECUTED

//! new-transaction
//! sender: alice
script{
use 0x0::ValidatorConfig;
// rotate alice's pubkey
fun main() {
    ValidatorConfig::rotate_consensus_pubkey_of_sender(x"40");
}
}

// check: events: []
// check: EXECUTED

// Run the block prologue. No reconfiguration should be triggered,
// since alice is not a validator
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
    ValidatorConfig::rotate_consensus_pubkey_of_sender(x"40");
    LibraSystem::update_and_reconfigure();
    // check that the validator set contains Vivian's new key after reconfiguration
    0x0::Transaction::assert(ValidatorConfig::get_consensus_pubkey(&LibraSystem::get_validator_config({{vivian}})) == x"40", 98);
}
}

// check: NewEpochEvent
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
    ValidatorConfig::rotate_consensus_pubkey_of_sender(x"40");
    LibraSystem::update_and_reconfigure();
}
}

// not: NewEpochEvent
// check: EXECUTED

//! block-prologue
//! proposer: vivian
//! block-time: 3

//! new-transaction
//! sender: viola
script{
use 0x0::LibraSystem;
use 0x0::ValidatorConfig;
// rotate viola's consensus pubkey to a new value
fun main() {
    ValidatorConfig::rotate_consensus_pubkey_of_sender(x"80");
    LibraSystem::update_and_reconfigure();
}
}

// check: NewEpochEvent
// check: EXECUTED

//! block-prologue
//! proposer: vivian
//! block-time: 4

//! new-transaction
//! sender: vivian
script{
use 0x0::LibraSystem;
use 0x0::ValidatorConfig;
// rotate vivian's consensus pubkey to a new value
fun main() {
    ValidatorConfig::rotate_consensus_pubkey_of_sender(x"90");
    LibraSystem::update_and_reconfigure();
}
}

// check: NewEpochEvent
// check: EXECUTED

//! new-transaction
//! sender: vivian
script{
use 0x0::LibraSystem;
use 0x0::ValidatorConfig;
// trigger a reconfiguration and check that both vivian and viola's key updates are reflected in
// the set
fun main() {
    // check that the validator set contains Viola's new key
    0x0::Transaction::assert(ValidatorConfig::get_consensus_pubkey(&LibraSystem::get_validator_config({{viola}})) == x"80", 15);
    // check that the validator set contains Vivian's new key
    0x0::Transaction::assert(ValidatorConfig::get_consensus_pubkey(&LibraSystem::get_validator_config({{vivian}})) == x"90", 17);
}
}
// check: EXECUTED
