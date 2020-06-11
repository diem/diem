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
        ValidatorConfig::set_config(account, {{bob}}, x"3d4017c3e843895a92b70aa74d1b7ebc9c982ccf2ec4968cc0cd55f12af4660c", x"", x"", x"", x"");

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
    use 0x1::ValidatorConfig;
    // test alice can invoke reconfiguration upon successful rotation of bob's consensus public key
    fun main(account: &signer) {
        ValidatorConfig::set_config(account, {{bob}}, x"d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a", x"", x"", x"", x"");
    }
}

// check: EXECUTED

//! new-transaction
//! sender: association
script {
    use 0x1::LibraSystem;
    use 0x1::Roles::{Self, AssociationRootRole};
    use 0x1::ValidatorConfig;
    // test alice can invoke reconfiguration upon successful rotation of bob's consensus public key
    fun main(account: &signer) {
        let assoc_root_role = Roles::extract_privilege_to_capability<AssociationRootRole>(account);
        // call update to reconfigure
        LibraSystem::update_and_reconfigure(&assoc_root_role);
        Roles::restore_capability_to_privilege(account, assoc_root_role);

        // check bob's public key is updated
        let validator_config = LibraSystem::get_validator_config({{bob}});
        assert(*ValidatorConfig::get_consensus_pubkey(&validator_config) == x"d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a", 99);

    }
}

// check: EXECUTED
