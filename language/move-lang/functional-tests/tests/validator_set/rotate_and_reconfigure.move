// Make sure bob can rotate his key locally, as
// well as call an update to change his
// key in the validator set and induce reconfiguration.

//! account: bob, 1000000, 0, validator

//! block-prologue
//! proposer: bob
//! block-time: 2

// check: EXECUTED

//! new-transaction
//! sender: bob
//! expiration-time: 3
// rotate bob's key with reconfiguration
script {
use 0x0::ValidatorConfig;
use 0x0::LibraSystem;
fun main(account: &signer) {
    // assert alice is a validator
    assert(ValidatorConfig::is_valid({{bob}}) == true, 98);
    assert(LibraSystem::is_validator({{bob}}) == true, 98);

    // bob rotates his public key
    ValidatorConfig::set_consensus_pubkey(account, {{bob}}, x"30");

    // use the update_token to trigger reconfiguration
    LibraSystem::update_and_reconfigure(account);

    // check bob's public key
    let validator_config = LibraSystem::get_validator_config({{bob}});
    assert(*ValidatorConfig::get_consensus_pubkey(&validator_config) == x"30", 99);
}
}

// check: NewEpochEvent
// check: EXECUTED
