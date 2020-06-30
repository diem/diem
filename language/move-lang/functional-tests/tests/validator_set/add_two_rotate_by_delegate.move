// Register alice as bob's delegate,
// test all possible key rotations:
// bob's key by bob - aborts
// bob's key by alice - executes
// alice's key by bob - aborts
// alice's key by alice - executes

//! account: alice, 1000000, 0, validator
//! account: bob, 1000000, 0, validator
//! account: carrol, 1000000, 0, validator

//! sender: bob
script {
    use 0x1::ValidatorConfig;
    fun main(account: &signer) {
        // set alice to change bob's key
        ValidatorConfig::set_operator(account, {{alice}});
    }
}

// check: EXECUTED

//! new-transaction
//! sender: bob
// check bob can not rotate his consensus key
script {
    use 0x1::ValidatorConfig;
    fun main(account: &signer) {
        ValidatorConfig::set_config(account, {{bob}}, x"d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a", x"", x"", x"", x"");
    }
}

// check: ABORTED

//! new-transaction
//! sender: bob
// check bob can not rotate alice's consensus key
script {
    use 0x1::ValidatorConfig;
    fun main(account: &signer) {
        ValidatorConfig::set_config(account, {{alice}}, x"d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a", x"", x"", x"", x"");
    }
}

// check: ABORTED

//! new-transaction
//! sender: alice
// check alice can rotate bob's consensus key
script {
    use 0x1::ValidatorConfig;
    fun main(account: &signer) {
        ValidatorConfig::set_config(account, {{bob}}, x"d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a", x"", x"", x"", x"");
        assert(*ValidatorConfig::get_consensus_pubkey(&ValidatorConfig::get_config({{bob}})) == x"d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a", 99);
    }
}

// check: EXECUTED

//! new-transaction
//! sender: alice
// check alice can rotate her consensus key
script {
    use 0x1::ValidatorConfig;
    fun main(account: &signer) {
        ValidatorConfig::set_config(account, {{alice}}, x"3d4017c3e843895a92b70aa74d1b7ebc9c982ccf2ec4968cc0cd55f12af4660c", x"", x"", x"", x"");
        assert(*ValidatorConfig::get_consensus_pubkey(&ValidatorConfig::get_config({{alice}})) == x"3d4017c3e843895a92b70aa74d1b7ebc9c982ccf2ec4968cc0cd55f12af4660c", 99);
    }
}

// check: EXECUTED

//! new-transaction
//! sender: alice
// check alice can rotate her consensus key
script {
    use 0x1::ValidatorConfig;
    fun main(account: &signer) {
        ValidatorConfig::set_config(account, {{alice}}, x"d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a", x"", x"", x"", x"");
        assert(*ValidatorConfig::get_consensus_pubkey(&ValidatorConfig::get_config({{alice}})) == x"d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a", 99);
    }
}

// check: EXECUTED
