// Make sure bob can rotate his key locally.
// An association may trigger bulk update to incorporate
// bob's key key into the validator set.

//! account: bob, 1000000, 0, validator

//! block-prologue
//! proposer: bob
//! block-time: 2

// check: EXECUTED

//! new-transaction
//! sender: bob
//! expiration-time: 3
// rotate bob's key
script {
    use 0x1::ValidatorConfig;
    use 0x1::LibraSystem;
    fun main(account: &signer) {
        // assert alice is a validator
        assert(ValidatorConfig::is_valid({{bob}}) == true, 98);
        assert(LibraSystem::is_validator({{bob}}) == true, 98);

        // bob rotates his public key
        ValidatorConfig::set_config(account, {{bob}},
                                    x"3d4017c3e843895a92b70aa74d1b7ebc9c982ccf2ec4968cc0cd55f12af4660c",
                                    x"", x"", x"", x"");
    }
}

// check: EXECUTED

//! new-transaction
//! sender: association
script {
    use 0x1::LibraSystem;
    use 0x1::ValidatorConfig;
    fun main(account: &signer) {
        // use the update_token to trigger reconfiguration
        LibraSystem::update_and_reconfigure(account);

        // check bob's public key
        let validator_config = LibraSystem::get_validator_config({{bob}});
        assert(*ValidatorConfig::get_consensus_pubkey(&validator_config) ==
               x"3d4017c3e843895a92b70aa74d1b7ebc9c982ccf2ec4968cc0cd55f12af4660c", 99);
    }
}

// check: NewEpochEvent
// check: EXECUTED
