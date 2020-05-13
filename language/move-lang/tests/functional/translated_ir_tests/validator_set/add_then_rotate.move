// Check that rotating keys between addition and reconfiguration works

//! account: alice
//! account: vivian, 1000000, 0, validator

//! sender: alice
// register Alice as a validator candidate
script{
use 0x0::ValidatorConfig;
fun main() {
    ValidatorConfig::register_candidate_validator(x"10", x"20", x"30", x"40", x"50", x"60");
}
}

// Bump the time so that a new reconfiguration can happen
//! block-prologue
//! proposer: vivian
//! block-time: 2

//! new-transaction
//! sender: association
// run a tx from the association that adds Alice as a validator
script{
use 0x0::LibraSystem;
fun main() {
    LibraSystem::add_validator({{alice}});
}
}

// check: NewEpochEvent
// check: EXECUTED

//! block-prologue
//! proposer: vivian
//! block-time: 3

//! new-transaction
//! sender: alice
// rotate alice's key
script{
use 0x0::LibraSystem;
use 0x0::ValidatorConfig;
fun main() {
    ValidatorConfig::rotate_consensus_pubkey_of_sender(x"beefbeef");
    LibraSystem::update_and_reconfigure();
}
}
// check: NewEpochEvent
// check: EXECUTED

//! new-transaction
// check that Alice is a validator AND that her key is as expected (i.e., it's the key after
// rotation, not the key before)
script{
use 0x0::LibraSystem;
use 0x0::ValidatorConfig;
fun main() {
    0x0::Transaction::assert(LibraSystem::is_validator({{alice}}), 70);
    0x0::Transaction::assert(ValidatorConfig::get_consensus_pubkey(&LibraSystem::get_validator_config({{alice}})) == x"beefbeef", 72);
}
}

// check: EXECUTED
