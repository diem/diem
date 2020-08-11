//! account: alice, 1000000, 0, validator
//! account: vivian, 1000000, 0, validator
//! account: viola, 1000000, 0, validator
//! account: bob, 0, 0, address
//! account: dave, 0, 0, address

//! block-prologue
//! proposer: vivian
//! block-time: 2

//! new-transaction
//! sender: libraroot
//! args: 0, {{bob}}, {{bob::auth_key}}, b"bob"
stdlib_script::create_validator_operator_account
// check: EXECUTED

//! new-transaction
//! sender: libraroot
//! args: 0, {{dave}}, {{dave::auth_key}}, b"dave"
stdlib_script::create_validator_operator_account
// check: EXECUTED

//! new-transaction
//! sender: alice
script {
    use 0x1::ValidatorConfig;
    fun main(account: &signer) {
        // set bob to be alice's operator
        ValidatorConfig::set_operator(account, {{bob}});
    }
}

// check: EXECUTED

//! new-transaction
//! sender: viola
script {
    use 0x1::ValidatorConfig;
    fun main(account: &signer) {
        // set dave to be viola's operator
        ValidatorConfig::set_operator(account, {{dave}});
    }
}

// check: EXECUTED

//! new-transaction
//! sender: libraroot
script{
    use 0x1::LibraSystem;
    // Decertify two validators to make sure we can remove both
    // from the set and trigger reconfiguration
    fun main(account: &signer) {
        assert(LibraSystem::is_validator({{alice}}) == true, 98);
        assert(LibraSystem::is_validator({{vivian}}) == true, 99);
        assert(LibraSystem::is_validator({{viola}}) == true, 100);
        LibraSystem::remove_validator(account, {{vivian}});
        assert(LibraSystem::is_validator({{alice}}) == true, 101);
        assert(LibraSystem::is_validator({{vivian}}) == false, 102);
        assert(LibraSystem::is_validator({{viola}}) == true, 103);
    }
}

// check: NewEpochEvent
// check: EXECUTED

//! block-prologue
//! proposer: alice
//! block-time: 3

// check: EXECUTED

//! new-transaction
//! sender: dave
script{
    use 0x1::LibraSystem;
    use 0x1::ValidatorConfig;
    // Two reconfigurations cannot happen in the same block
    fun main(account: &signer) {
        // the local validator's key was the same as the key in the validator set
        assert(ValidatorConfig::get_consensus_pubkey(&LibraSystem::get_validator_config({{viola}})) ==
               ValidatorConfig::get_consensus_pubkey(&ValidatorConfig::get_config({{viola}})), 99);
        ValidatorConfig::set_config(account, {{viola}},
                                    x"d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a",
                                    x"", x"", x"", x"");
        // the local validator's key is now different from the one in the validator set
        assert(ValidatorConfig::get_consensus_pubkey(&LibraSystem::get_validator_config({{viola}})) !=
               ValidatorConfig::get_consensus_pubkey(&ValidatorConfig::get_config({{viola}})), 99);
        let old_num_validators = LibraSystem::validator_set_size();
        LibraSystem::update_config_and_reconfigure(account, {{viola}});
        assert(old_num_validators == LibraSystem::validator_set_size(), 98);
        // the local validator's key is now the same as the key in the validator set
        assert(ValidatorConfig::get_consensus_pubkey(&LibraSystem::get_validator_config({{viola}})) ==
               ValidatorConfig::get_consensus_pubkey(&ValidatorConfig::get_config({{viola}})), 99);
    }
}

// check: NewEpochEvent
// check: EXECUTED

//! new-transaction
//! sender: dave
script{
    use 0x1::LibraSystem;
    use 0x1::ValidatorConfig;
    fun main(account: &signer) {
        ValidatorConfig::set_config(account, {{viola}},
                                    x"3d4017c3e843895a92b70aa74d1b7ebc9c982ccf2ec4968cc0cd55f12af4660c",
                                    x"", x"", x"", x"");
        let old_num_validators = LibraSystem::validator_set_size();
        LibraSystem::update_config_and_reconfigure(account, {{viola}});
        assert(old_num_validators == LibraSystem::validator_set_size(), 98);
    }
}

// check: "Keep(ABORTED { code: 1025,"

//! new-transaction
//! sender: bob
script{
    use 0x1::LibraSystem;
    fun main(account: &signer) {
        LibraSystem::update_config_and_reconfigure(account, {{viola}});
    }
}

// check: "Keep(ABORTED { code: 1031,"

//! new-transaction
//! sender: blessed
// freezing does not cause changes to the set
script {
    use 0x1::LibraSystem;
    use 0x1::AccountFreezing;
    fun main(tc_account: &signer) {
        assert(LibraSystem::is_validator({{alice}}) == true, 101);
        AccountFreezing::freeze_account(tc_account, {{alice}});
        assert(AccountFreezing::account_is_frozen({{alice}}), 1);
        assert(LibraSystem::is_validator({{alice}}) == true, 102);
    }
}
// check: EXECUTED
