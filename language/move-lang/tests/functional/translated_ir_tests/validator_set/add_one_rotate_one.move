// Add bob as a validator.
// Make sure bob can rotate his key locally, as
// well as call force-update to change his
// key in the validator set and induce reconfiguration.

//! account: alice
//! account: bob, 1000000, 0, validator

//! sender: bob
use 0x0::ValidatorConfig2;
fun main() {
    ValidatorConfig2::initialize(x"beefbeef");
}

// check: EXECUTED

//! block-prologue
//! proposer: bob
//! block-time: 2

// check: EXECUTED

//! new-transaction
//! sender: bob
// rotate bob's key without reconfiguration
use 0x0::ValidatorConfig2;
fun main() {
    let config = ValidatorConfig2::get_config({{bob}});
    0x0::Transaction::assert(ValidatorConfig2::get_consensus_pubkey(&config) == x"beefbeef", 99);
    ValidatorConfig2::rotate_consensus_pubkey_of_sender(x"20", x"10");
    config = ValidatorConfig2::get_config({{bob}});
    0x0::Transaction::assert(ValidatorConfig2::get_consensus_pubkey(&config) == x"20", 99);
}

// check: EXECUTED

//! new-transaction
//! sender: association
use 0x0::LibraSystem2;
use 0x0::ValidatorConfig2;
fun main() {
    // add validator
    LibraSystem2::add_validator({{bob}});

    // check bob's public key
    let validator_info = LibraSystem2::get_validator_info({{bob}});
    let validator_config = LibraSystem2::get_validator_config(&validator_info);
    0x0::Transaction::assert(ValidatorConfig2::get_consensus_pubkey(validator_config) == x"20", 99);
}

// check: NewEpochEvent
// check: EXECUTED

//! block-prologue
//! proposer: bob
//! block-time: 86400000005

// check: EXECUTED

//! new-transaction
//! sender: bob
//! expiration-time: 864000000000
// rotate bob's key with reconfiguration
use 0x0::ValidatorConfig2;
use 0x0::LibraSystem2;
fun main() {
    // check bob's public key
    let validator_info = LibraSystem2::get_validator_info({{bob}});
    let validator_config = LibraSystem2::get_validator_config(&validator_info);
    0x0::Transaction::assert(ValidatorConfig2::get_consensus_pubkey(validator_config) == x"20", 99);

    // bob rotates his public key
    ValidatorConfig2::rotate_consensus_pubkey_of_sender(x"30", x"10");

    // use the update_token to trigger reconfiguration
    LibraSystem2::force_update_validator_info({{bob}});

    // check bob's public key
    validator_info = LibraSystem2::get_validator_info({{bob}});
    validator_config = LibraSystem2::get_validator_config(&validator_info);
    0x0::Transaction::assert(ValidatorConfig2::get_consensus_pubkey(validator_config) == x"30", 99);
}

// check: NewEpochEvent
// check: EXECUTED
