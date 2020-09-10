// Register alice as bob's delegate
// test all possible key rotations:
// bob's key by bob - aborts
// bob's key by alice - executes
// alice's key by bob - aborts
// alice's key by alice - executes

//! account: alice, 0, 0, address
//! account: bob, 1000000, 0, validator
//! account: carrol, 1000000, 0, validator

//! new-transaction
//! sender: libraroot
//! args: 0, {{alice}}, {{alice::auth_key}}, b"alice"
stdlib_script::create_validator_operator_account
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: bob
script {
    use 0x1::ValidatorConfig;
    fun main(account: &signer) {
        // set alice to change bob's key
        ValidatorConfig::set_operator(account, {{alice}});
    }
}

// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: bob
// check bob can not rotate his consensus key
script {
    use 0x1::ValidatorConfig;
    fun main(account: &signer) {
        ValidatorConfig::set_config(account, {{bob}}, x"d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a", x"", x"");
    }
}

// check: "Keep(ABORTED { code: 263,"

//! new-transaction
//! sender: bob
// check bob can not rotate alice's consensus key
script {
    use 0x1::ValidatorConfig;
    fun main(account: &signer) {
        ValidatorConfig::set_config(account, {{alice}}, x"d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a", x"", x"");
    }
}

// check: "Keep(ABORTED { code: 5,"

//! new-transaction
//! sender: alice
// check alice can rotate bob's consensus key
script {
    use 0x1::ValidatorConfig;
    fun main(account: &signer) {
        ValidatorConfig::set_config(account, {{bob}}, x"d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a", x"", x"");
        assert(*ValidatorConfig::get_consensus_pubkey(&ValidatorConfig::get_config({{bob}})) == x"d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a", 99);
    }
}

// check: "Keep(EXECUTED)"
