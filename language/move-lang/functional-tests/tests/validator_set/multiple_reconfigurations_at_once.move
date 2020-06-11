//! account: alice, 1000000, 0, validator
//! account: vivian, 1000000, 0, validator
//! account: viola, 1000000, 0, validator

//! block-prologue
//! proposer: vivian
//! block-time: 2

// check: EXECUTED

//! new-transaction
//! sender: association
script{
    use 0x1::LibraSystem;
    use 0x1::Roles::{Self, AssociationRootRole};
    // Decertify two validators to make sure we can remove both
    // from the set and trigger reconfiguration
    fun main(account: &signer) {
        assert(LibraSystem::is_validator({{alice}}) == true, 98);
        assert(LibraSystem::is_validator({{vivian}}) == true, 99);
        assert(LibraSystem::is_validator({{viola}}) == true, 100);
        let assoc_root_role = Roles::extract_privilege_to_capability<AssociationRootRole>(account);
        LibraSystem::remove_validator(&assoc_root_role, {{vivian}});
        Roles::restore_capability_to_privilege(account, assoc_root_role);
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
//! sender: viola
script{
    use 0x1::ValidatorConfig;
    // Two reconfigurations cannot happen in the same block
    fun main(account: &signer) {
        ValidatorConfig::set_config(account, {{viola}},
                                    x"d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a",
                                    x"", x"", x"", x"");
    }
}

// check: EXECUTED

//! new-transaction
//! sender: association
script{
    use 0x1::LibraSystem;
    use 0x1::Roles::{Self, AssociationRootRole};
    fun main(account: &signer) {
        let assoc_root_role = Roles::extract_privilege_to_capability<AssociationRootRole>(account);
        LibraSystem::update_and_reconfigure(&assoc_root_role);
        Roles::restore_capability_to_privilege(account, assoc_root_role);
    }
}

// check: NewEpochEvent
// check: EXECUTED

//! new-transaction
//! sender: viola
script{
    use 0x1::ValidatorConfig;
    fun main(account: &signer) {
        ValidatorConfig::set_config(account, {{viola}},
                                    x"3d4017c3e843895a92b70aa74d1b7ebc9c982ccf2ec4968cc0cd55f12af4660c",
                                    x"", x"", x"", x"");
    }
}

// check: EXECUTED

//! new-transaction
//! sender: association
script{
    use 0x1::LibraSystem;
    use 0x1::Roles::{Self, AssociationRootRole};
    fun main(account: &signer) {
        let assoc_root_role = Roles::extract_privilege_to_capability<AssociationRootRole>(account);
        LibraSystem::update_and_reconfigure(&assoc_root_role);
        Roles::restore_capability_to_privilege(account, assoc_root_role);
    }
}

// check: ABORT
// check: 23
