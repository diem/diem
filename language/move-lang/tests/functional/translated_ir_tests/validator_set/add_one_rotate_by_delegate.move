// Make bob a validator, set alice as bob's delegate.
// Test that alice can rotate bob's key and invoke reconfiguration.

//! account: alice
//! account: bob, 1000000, 0, validator

//! sender: bob
use 0x0::ValidatorConfig2;
fun main() {
    ValidatorConfig2::initialize(x"beefbeef", x"10", x"20", x"30", x"40", x"50");
    // register alice as bob's delegate
    ValidatorConfig2::set_delegated_account({{alice}});
}

// check: EXECUTED

//! new-transaction
//! sender: alice
use 0x0::ValidatorConfig2;
// test alice can rotate bob's consensus public key
fun main() {
    // check initial key was "beefbeef"
    let config = ValidatorConfig2::get_config({{bob}});
    0x0::Transaction::assert(ValidatorConfig2::get_consensus_pubkey(&config) == x"beefbeef", 99);

    0x0::Transaction::assert(ValidatorConfig2::get_validator_operator_account({{bob}}) == {{alice}}, 44);
    ValidatorConfig2::rotate_consensus_pubkey({{bob}}, x"20", x"10");

    // check new key is "20"
    config = ValidatorConfig2::get_config({{bob}});
    0x0::Transaction::assert(ValidatorConfig2::get_consensus_pubkey(&config) == x"20", 99);
}

// check: EXECUTED

//! new-transaction
//! sender: bob
use 0x0::ValidatorConfig2;
// test bob can not rotate his public key because it delegated
fun main() {
    // check initial key was "beefbeef"
    let config = ValidatorConfig2::get_config({{bob}});
    0x0::Transaction::assert(ValidatorConfig2::get_consensus_pubkey(&config) == x"20", 99);

    ValidatorConfig2::rotate_consensus_pubkey_of_sender(x"30", x"10");
}

// check: ABORTED

//! block-prologue
//! proposer: bob
//! block-time: 2

// check: EXECUTED

//! new-transaction
//! sender: association
use 0x0::LibraSystem2;
fun main() {
    // add validator
    LibraSystem2::add_validator({{bob}});
}

// check: NewEpochEvent
// check: EXECUTED

//! block-prologue
//! proposer: bob
//! block-time: 86400000010

// check: EXECUTED

//! new-transaction
//! sender: alice
//! expiration-time: 864000000020
// after >= 24hr has passed since bob was added as a validator with a current
// consensus pubkey, he can call a force_update to the validator_set (due to
// rate limits, 24hr = 86_400_000_000us
// to test that the block-time is set to 24hr + 10us and the expiration time
// of the transaction is made sufficiently large
use 0x0::ValidatorConfig2;
use 0x0::LibraSystem2;
// test alice can invoke reconfiguration upon successful rotation of bob's consensus public key
fun main() {
    // check initial key was "20"
    let validator_info = LibraSystem2::get_validator_info({{bob}});
    let validator_config = LibraSystem2::get_validator_config(&validator_info);
    0x0::Transaction::assert(ValidatorConfig2::get_consensus_pubkey(validator_config) == x"20", 99);

    ValidatorConfig2::rotate_consensus_pubkey({{bob}}, x"30", x"10");

    // call force_update to reconfigure
    LibraSystem2::force_update_validator_info({{bob}});

    // check bob's public key is updated
    validator_info = LibraSystem2::get_validator_info({{bob}});
    validator_config = LibraSystem2::get_validator_config(&validator_info);
    0x0::Transaction::assert(ValidatorConfig2::get_consensus_pubkey(validator_config) == x"30", 99);
}

// check: NewEpochEvent
// check: EXECUTED
