// Make bob a validator, set alice as bob's delegate.
// Test that alice can rotate bob's key and invoke reconfiguration.

//! account: alice
//! account: bob, 1000000, 0, validator
//! account: carrol, 1000000, 0, validator

//! sender: bob
script {
    use 0x1::ValidatorConfig;
    fun main(account: &signer) {
        // register alice as bob's delegate
        ValidatorConfig::set_operator(account, {{alice}});
    }
}

// check: EXECUTED

//! new-transaction
//! sender: alice
script {
    use 0x1::ValidatorConfig;
    // test alice can rotate bob's consensus public key
    fun main(account: &signer) {
        assert(ValidatorConfig::get_operator({{bob}}) == {{alice}}, 44);
        ValidatorConfig::set_config(account, {{bob}},
                                    x"3d4017c3e843895a92b70aa74d1b7ebc9c982ccf2ec4968cc0cd55f12af4660c",
                                    x"", x"", x"", x"");

        // check new key is "20"
        let config = ValidatorConfig::get_config({{bob}});
        assert(*ValidatorConfig::get_consensus_pubkey(&config) == x"3d4017c3e843895a92b70aa74d1b7ebc9c982ccf2ec4968cc0cd55f12af4660c", 99);
    }
}

// check: EXECUTED

//! new-transaction
//! sender: bob
script {
    use 0x1::ValidatorConfig;
    // test bob can not rotate his public key because it delegated
    fun main(account: &signer) {
        // check initial key was "beefbeef"
        let config = ValidatorConfig::get_config({{bob}});
        assert(*ValidatorConfig::get_consensus_pubkey(&config) == x"3d4017c3e843895a92b70aa74d1b7ebc9c982ccf2ec4968cc0cd55f12af4660c", 99);

        ValidatorConfig::set_config(account, {{bob}}, x"d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a", x"", x"", x"", x"");
    }
}

// check: ABORTED

//! block-prologue
//! proposer: carrol
//! block-time: 2

// check: EXECUTED

//! new-transaction
//! sender: alice
//! expiration-time: 3
script {
    use 0x1::LibraSystem;
    use 0x1::ValidatorConfig;
    fun main(account: &signer) {
        ValidatorConfig::set_config(account, {{bob}}, x"d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a", x"", x"", x"", x"");
        // the local validator's key is now different from the one in the validator set
        assert(ValidatorConfig::get_consensus_pubkey(&LibraSystem::get_validator_config({{bob}})) !=
               ValidatorConfig::get_consensus_pubkey(&ValidatorConfig::get_config({{bob}})), 99);
        LibraSystem::update_config_and_reconfigure(account, {{bob}});
        // the local validator's key is now the same as the key in the validator set
        assert(ValidatorConfig::get_consensus_pubkey(&LibraSystem::get_validator_config({{bob}})) ==
               ValidatorConfig::get_consensus_pubkey(&ValidatorConfig::get_config({{bob}})), 99);
        // check bob's public key is updated
        let validator_config = LibraSystem::get_validator_config({{bob}});
        assert(*ValidatorConfig::get_consensus_pubkey(&validator_config) == x"d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a", 99);
    }
}

// check: NewEpochEvent
// check: EXECUTED
