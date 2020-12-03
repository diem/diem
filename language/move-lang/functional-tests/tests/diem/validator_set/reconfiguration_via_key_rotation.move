//! account: alice, 1000000, 0, validator
//! account: bob, 0, 0, address
//! account: vivian, 1000000, 0, validator
//! account: dave, 0, 0, address
//! account: viola, 1000000, 0, validator

//! new-transaction
//! sender: diemroot
//! args: 0, {{bob}}, {{bob::auth_key}}, b"bob"
stdlib_script::create_validator_operator_account
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: diemroot
//! args: 0, {{dave}}, {{dave::auth_key}}, b"dave"
stdlib_script::create_validator_operator_account
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: alice
script {
    use 0x1::ValidatorConfig;
    fun main(account: &signer) {
        // set bob to change alice's key
        ValidatorConfig::set_operator(account, {{bob}});
    }
}

// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: vivian
script {
    use 0x1::ValidatorConfig;
    fun main(account: &signer) {
        // set dave to change vivian's key
        ValidatorConfig::set_operator(account, {{dave}});
    }
}

// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: bob
script{
    use 0x1::ValidatorConfig;
    // rotate alice's pubkey
    fun main(account: &signer) {
        ValidatorConfig::set_config(account, {{alice}}, x"d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a", x"", x"");
    }
}

// check: events: []
// check: "Keep(EXECUTED)"

//! block-prologue
//! proposer: vivian
//! block-time: 300000001

// not: NewEpochEvent
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: dave
script{
    use 0x1::DiemSystem;
    use 0x1::ValidatorConfig;
    // rotate vivian's pubkey and then run the block prologue. Now, reconfiguration should be triggered.
    fun main(account: &signer) {
        assert(*ValidatorConfig::get_consensus_pubkey(&DiemSystem::get_validator_config({{vivian}})) !=
               x"d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a", 98);
        ValidatorConfig::set_config(account, {{vivian}}, x"d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a", x"", x"");
        DiemSystem::update_config_and_reconfigure(account, {{vivian}});
        // check that the validator set contains Vivian's new key after reconfiguration
        assert(*ValidatorConfig::get_consensus_pubkey(&DiemSystem::get_validator_config({{vivian}})) ==
               x"d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a", 99);
    }
}

// check: NewEpochEvent
// check: "Keep(EXECUTED)"

//! block-prologue
//! proposer: vivian
//! block-time: 600000002

// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: dave
script{
    use 0x1::DiemSystem;
    use 0x1::ValidatorConfig;
    // rotate vivian's pubkey to the same value does not trigger the reconfiguration.
    fun main(account: &signer) {
        ValidatorConfig::set_config(account, {{vivian}}, x"d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a", x"", x"");
        DiemSystem::update_config_and_reconfigure(account, {{vivian}});
    }
}

// not: NewEpochEvent
// check: "Keep(EXECUTED)"
