// Make sure bob can rotate his key locally.
// The diem root account may trigger bulk update to incorporate
// bob's key key into the validator set.

//! account: alice, 0, 0, address
//! account: bob, 0, 0, validator

//! new-transaction
//! sender: diemroot
//! args: 0, {{alice}}, {{alice::auth_key}}, b"alice"
stdlib_script::AccountCreationScripts::create_validator_operator_account
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: bob
script {
    use 0x1::ValidatorConfig;
    use 0x1::DiemSystem;
    fun main(account: signer) {
    let account = &account;
        // register alice as bob's delegate
        ValidatorConfig::set_operator(account, {{alice}});

        // assert bob is a validator
        assert(ValidatorConfig::is_valid({{bob}}) == true, 98);
        assert(DiemSystem::is_validator({{bob}}) == true, 98);
    }
}

// check: "Keep(EXECUTED)"

//! block-prologue
//! proposer: bob
//! block-time: 300000001

// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: alice
// rotate bob's key
script {
    use 0x1::DiemSystem;
    use 0x1::ValidatorConfig;
    fun main(account: signer) {
    let account = &account;
        // assert bob is a validator
        assert(ValidatorConfig::is_valid({{bob}}) == true, 98);
        assert(DiemSystem::is_validator({{bob}}) == true, 98);

        assert(ValidatorConfig::get_consensus_pubkey(&DiemSystem::get_validator_config({{bob}})) ==
               ValidatorConfig::get_consensus_pubkey(&ValidatorConfig::get_config({{bob}})), 99);

        // alice rotates bob's public key
        ValidatorConfig::set_config(account, {{bob}},
                                    x"3d4017c3e843895a92b70aa74d1b7ebc9c982ccf2ec4968cc0cd55f12af4660c",
                                    x"", x"");
        DiemSystem::update_config_and_reconfigure(account, {{bob}});
        // check bob's public key
        let validator_config = DiemSystem::get_validator_config({{bob}});
        assert(*ValidatorConfig::get_consensus_pubkey(&validator_config) ==
               x"3d4017c3e843895a92b70aa74d1b7ebc9c982ccf2ec4968cc0cd55f12af4660c", 99);
    }
}

// check: NewEpochEvent
// check: "Keep(EXECUTED)"
