// Make bob a validator, set alice as bob's delegate.
// Test that alice can rotate bob's key and invoke reconfiguration.

//! account: alice
//! account: bob
//! account: carrol, 1000000, 0, validator

//! sender: bob
script {
use 0x0::ValidatorConfig;
fun main() {
    ValidatorConfig::register_candidate_validator(x"beefbeef", x"10", x"20", x"30", x"40", x"50");
    // register alice as bob's delegate
    ValidatorConfig::set_delegated_account({{alice}});
}
}

// check: EXECUTED

//! new-transaction
//! sender: alice
script {
use 0x0::ValidatorConfig;
// test alice can rotate bob's consensus public key
fun main() {
    0x0::Transaction::assert(ValidatorConfig::get_validator_operator_account({{bob}}) == {{alice}}, 44);
    ValidatorConfig::rotate_consensus_pubkey({{bob}}, x"20");

    // check new key is "20"
    let config = ValidatorConfig::get_config({{bob}});
    0x0::Transaction::assert(ValidatorConfig::get_consensus_pubkey(&config) == x"20", 99);
}
}

// check: EXECUTED

//! new-transaction
//! sender: bob
script {
use 0x0::ValidatorConfig;
// test bob can not rotate his public key because it delegated
fun main() {
    // check initial key was "beefbeef"
    let config = ValidatorConfig::get_config({{bob}});
    0x0::Transaction::assert(ValidatorConfig::get_consensus_pubkey(&config) == x"20", 99);

    ValidatorConfig::rotate_consensus_pubkey_of_sender(x"30");
}
}

// check: ABORTED

//! block-prologue
//! proposer: carrol
//! block-time: 2

// check: EXECUTED

//! new-transaction
//! sender: association
script {
use 0x0::LibraSystem;
fun main() {
    // add validator
    LibraSystem::add_validator({{bob}});
}
}

// check: EXECUTED

//! block-prologue
//! proposer: carrol
//! block-time: 86400000010

// check: EXECUTED

//! new-transaction
//! sender: alice
//! expiration-time: 864000000020
// after >= 24hr has passed since bob was added as a validator with a current
// consensus pubkey, he can call an update to the validator_set (due to
// rate limits, 24hr = 86_400_000_000us
// to test that the block-time is set to 24hr + 10us and the expiration time
// of the transaction is made sufficiently large
script {
use 0x0::ValidatorConfig;
use 0x0::LibraSystem;
// test alice can invoke reconfiguration upon successful rotation of bob's consensus public key
fun main() {
    // check initial key was "20"
    let validator_config = LibraSystem::get_validator_config({{bob}});
    0x0::Transaction::assert(ValidatorConfig::get_consensus_pubkey(&validator_config) == x"20", 99);

    ValidatorConfig::rotate_consensus_pubkey({{bob}}, x"30");

    // call update to reconfigure
    LibraSystem::update_and_reconfigure();

    // check bob's public key is updated
    validator_config = LibraSystem::get_validator_config({{bob}});
    0x0::Transaction::assert(ValidatorConfig::get_consensus_pubkey(&validator_config) == x"30", 99);
}
}

// check: EXECUTED
