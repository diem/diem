// Make sure bob can rotate his key locally.
// The libra root account may trigger bulk update to incorporate
// bob's key key into the validator set.

//! account: alice, 0, 0, address
//! account: bob, 0, 0, validator

//! new-transaction
//! sender: libraroot
//! args: 0, {{alice}}, {{alice::auth_key}}, b"alice"
stdlib_script::create_validator_operator_account
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: bob
script {
    use 0x1::ValidatorConfig;
    use 0x1::LibraSystem;
    fun main(account: &signer) {
        // register alice as bob's delegate
        ValidatorConfig::set_operator(account, {{alice}});

        // assert bob is a validator
        assert(ValidatorConfig::is_valid({{bob}}) == true, 98);
        assert(LibraSystem::is_validator({{bob}}) == true, 98);
    }
}

// check: "Keep(EXECUTED)"

//! block-prologue
//! proposer: bob
//! block-time: 2

// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: alice
//! expiration-time: 3
// rotate bob's key
script {
    use 0x1::LibraSystem;
    use 0x1::ValidatorConfig;
    fun main(account: &signer) {
        // assert bob is a validator
        assert(ValidatorConfig::is_valid({{bob}}) == true, 98);
        assert(LibraSystem::is_validator({{bob}}) == true, 98);

        assert(ValidatorConfig::get_consensus_pubkey(&LibraSystem::get_validator_config({{bob}})) ==
               ValidatorConfig::get_consensus_pubkey(&ValidatorConfig::get_config({{bob}})), 99);

        // alice rotates bob's public key
        ValidatorConfig::set_config(account, {{bob}},
                                    x"3d4017c3e843895a92b70aa74d1b7ebc9c982ccf2ec4968cc0cd55f12af4660c",
                                    x"", x"");
        LibraSystem::update_config_and_reconfigure(account, {{bob}});
        // check bob's public key
        let validator_config = LibraSystem::get_validator_config({{bob}});
        assert(*ValidatorConfig::get_consensus_pubkey(&validator_config) ==
               x"3d4017c3e843895a92b70aa74d1b7ebc9c982ccf2ec4968cc0cd55f12af4660c", 99);
    }
}

// check: NewEpochEvent
// check: "Keep(EXECUTED)"
