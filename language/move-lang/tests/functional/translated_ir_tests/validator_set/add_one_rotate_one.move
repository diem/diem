// Add bob as a validator.
// Make sure bob can rotate his key locally, as
// well as call an update to change his
// key in the validator set and induce reconfiguration.

//! account: alice
//! account: bob, 1000000, 0, validator

//! sender: bob
use 0x0::ValidatorConfig;
fun main() {
    // test bob is a validator
    0x0::Transaction::assert(ValidatorConfig::has({{bob}}) == true, 98);
}

// check: EXECUTED

//! block-prologue
//! proposer: bob
//! block-time: 86400000005

// check: EXECUTED

//! new-transaction
//! sender: bob
//! expiration-time: 864000000000
// rotate bob's key with reconfiguration
use 0x0::ValidatorConfig;
use 0x0::LibraSystem;
fun main() {
    // check bob's public key
    // let validator_info = LibraSystem::get_validator_info(&{{bob}});
    // let validator_config = LibraSystem::get_validator_config(&validator_info);

    // bob rotates his public key
    ValidatorConfig::rotate_consensus_pubkey_of_sender(x"30");

    // use the update_token to trigger reconfiguration
    LibraSystem::update_and_reconfigure();

    // check bob's public key
    let validator_info = LibraSystem::get_validator_info({{bob}});
    let validator_config = LibraSystem::get_validator_config(&validator_info);
    0x0::Transaction::assert(ValidatorConfig::get_consensus_pubkey(validator_config) == x"30", 99);
}

// check: EXECUTED
