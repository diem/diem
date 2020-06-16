//! account: alice
//! account: vivian, 1000000, 0, validator
//! account: v1, 1000000, 0, validator
//! account: v2, 1000000, 0, validator
//! account: v3, 1000000, 0, validator

//! block-prologue
//! proposer: vivian
//! block-time: 3

//! new-transaction
//! sender: association
// remove_validator cannot be called on a non-validator
script{
    use 0x1::LibraSystem;
    use 0x1::Roles::{Self, AssociationRootRole};
    fun main(account: &signer) {
        let assoc_root_role = Roles::extract_privilege_to_capability<AssociationRootRole>(account);
        LibraSystem::remove_validator(&assoc_root_role, {{alice}});
        Roles::restore_capability_to_privilege(account, assoc_root_role);
    }
}

// check: ABORTED
// check: 21

// remove_validator can only be called by the Association
//! new-transaction
//! sender: alice
script{
    use 0x1::LibraSystem;
    use 0x1::Roles::{Self, AssociationRootRole};
    fun main(account: &signer) {
        let assoc_root_role = Roles::extract_privilege_to_capability<AssociationRootRole>(account);
        LibraSystem::remove_validator(&assoc_root_role, {{vivian}});
        Roles::restore_capability_to_privilege(account, assoc_root_role);
    }
}

// check: ABORTED

//! new-transaction
//! sender: association
// should work because Vivian is a validator
script{
    use 0x1::LibraSystem;
    use 0x1::Roles::{Self, AssociationRootRole};
    fun main(account: &signer) {
        let assoc_root_role = Roles::extract_privilege_to_capability<AssociationRootRole>(account);
        LibraSystem::remove_validator(&assoc_root_role, {{vivian}});
        Roles::restore_capability_to_privilege(account, assoc_root_role);
    }
}

// check: NewEpochEvent
// check: EXECUTED

//! new-transaction
//! sender: association
// double-removing Vivian should fail
script{
    use 0x1::LibraSystem;
    use 0x1::Roles::{Self, AssociationRootRole};
    fun main(account: &signer) {
        let assoc_root_role = Roles::extract_privilege_to_capability<AssociationRootRole>(account);
        LibraSystem::remove_validator(&assoc_root_role, {{vivian}});
        Roles::restore_capability_to_privilege(account, assoc_root_role);
    }
}

// check: ABORTED
