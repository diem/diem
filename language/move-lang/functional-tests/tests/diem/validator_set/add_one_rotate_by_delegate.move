// Make bob a validator, set alice as bob's delegate.
// Test that alice can rotate bob's key and invoke reconfiguration.

//! account: alice, 0, 0, address
//! account: bob, 1000000, 0, validator
//! account: carrol, 1000000, 0, validator

//! new-transaction
//! sender: diemroot
//! args: 0, {{alice}}, {{alice::auth_key}}, b"alice"
stdlib_script::AccountCreationScripts::create_validator_operator_account
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: bob
script {
    use DiemFramework::ValidatorConfig;
    fun main(account: signer) {
    let account = &account;
        // register alice as bob's delegate
        ValidatorConfig::set_operator(account, @{{alice}});
    }
}

// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: alice
script {
    use DiemFramework::ValidatorConfig;
    // test alice can rotate bob's consensus public key
    fun main(account: signer) {
    let account = &account;
        assert(ValidatorConfig::get_operator(@{{bob}}) == @{{alice}}, 44);
        ValidatorConfig::set_config(account, @{{bob}}, x"3d4017c3e843895a92b70aa74d1b7ebc9c982ccf2ec4968cc0cd55f12af4660c", x"", x"");

        // check new key is "20"
        let config = ValidatorConfig::get_config(@{{bob}});
        assert(*ValidatorConfig::get_consensus_pubkey(&config) == x"3d4017c3e843895a92b70aa74d1b7ebc9c982ccf2ec4968cc0cd55f12af4660c", 99);
    }
}

// check: "Keep(EXECUTED)"

//! block-prologue
//! proposer: carrol
//! block-time: 300000001

// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: alice
script {
    use DiemFramework::DiemSystem;
    use DiemFramework::ValidatorConfig;
    fun main(account: signer) {
    let account = &account;
        ValidatorConfig::set_config(account, @{{bob}}, x"d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a", x"", x"");
        // the local validator's key is now different from the one in the validator set
        assert(ValidatorConfig::get_consensus_pubkey(&DiemSystem::get_validator_config(@{{bob}})) !=
               ValidatorConfig::get_consensus_pubkey(&ValidatorConfig::get_config(@{{bob}})), 99);
        DiemSystem::update_config_and_reconfigure(account, @{{bob}});
        // the local validator's key is now the same as the key in the validator set
        assert(ValidatorConfig::get_consensus_pubkey(&DiemSystem::get_validator_config(@{{bob}})) ==
               ValidatorConfig::get_consensus_pubkey(&ValidatorConfig::get_config(@{{bob}})), 99);
        // check bob's public key is updated
        let validator_config = DiemSystem::get_validator_config(@{{bob}});
        assert(*ValidatorConfig::get_consensus_pubkey(&validator_config) == x"d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a", 99);
    }
}

// check: NewEpochEvent
// check: "Keep(EXECUTED)"
