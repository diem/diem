// Check that removing a non-existent validator aborts.

//! account: alice
//! account: bob, 1000000, 0, validator

//! sender: alice
script {
    use 0x1::LibraSystem;
    use 0x1::Roles::{Self, LibraRootRole};
    fun main(account: &signer) {
        // alice cannot remove herself
        let assoc_root_role = Roles::extract_privilege_to_capability<LibraRootRole>(account);
        LibraSystem::remove_validator(&assoc_root_role, {{alice}});
        Roles::restore_capability_to_privilege(account, assoc_root_role);
    }
}

// check: ABORTED

//! new-transaction
//! sender: alice
script {
    use 0x1::LibraSystem;
    use 0x1::Roles::{Self, LibraRootRole};
    fun main(account: &signer) {
        // alice cannot remove bob
        let assoc_root_role = Roles::extract_privilege_to_capability<LibraRootRole>(account);
        LibraSystem::remove_validator(&assoc_root_role, {{bob}});
        Roles::restore_capability_to_privilege(account, assoc_root_role);
    }
}

// check: ABORTED

//! new-transaction
//! sender: bob
script {
    use 0x1::LibraSystem;
    use 0x1::Roles::{Self, LibraRootRole};
    fun main(account: &signer) {
        // bob cannot remove alice
        let assoc_root_role = Roles::extract_privilege_to_capability<LibraRootRole>(account);
        LibraSystem::remove_validator(&assoc_root_role, {{alice}});
        Roles::restore_capability_to_privilege(account, assoc_root_role);
    }
}

// check: ABORTED
