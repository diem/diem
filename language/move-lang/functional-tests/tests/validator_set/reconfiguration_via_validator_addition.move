//! account: alice, 1000000, 0, validator
//! account: bob, 1000000, 0, validator

//! block-prologue
//! proposer: bob
//! block-time: 2

// check: EXECUTED

//! new-transaction
//! sender: association
script{
    use 0x1::LibraSystem;
    use 0x1::Roles::{Self, LibraRootRole};
    fun main(account: &signer) {
        let assoc_root_role = Roles::extract_privilege_to_capability<LibraRootRole>(account);
        LibraSystem::remove_validator(&assoc_root_role, {{alice}});
        Roles::restore_capability_to_privilege(account, assoc_root_role);
        assert(!LibraSystem::is_validator({{alice}}), 77);
        assert(LibraSystem::is_validator({{bob}}), 78);
    }
}
// check: NewEpochEvent
// check: EXECUTED

//! block-prologue
//! proposer: bob
//! block-time: 3

// check: EXECUTED

//! new-transaction
//! sender: bob
// bob cannot remove itself, only the association can remove validators from the set
script{
    use 0x1::LibraSystem;
    use 0x1::Roles::{Self, LibraRootRole};
    fun main(account: &signer) {
        let assoc_root_role = Roles::extract_privilege_to_capability<LibraRootRole>(account);
        LibraSystem::remove_validator(&assoc_root_role, {{bob}});
        Roles::restore_capability_to_privilege(account, assoc_root_role);
    }
}
// check: ABORTED

//! block-prologue
//! proposer: bob
//! block-time: 4

// check: EXECUTED

//! new-transaction
//! sender: association
script{
    use 0x1::LibraSystem;
    use 0x1::Roles::{Self, LibraRootRole};
    fun main(account: &signer) {
        let assoc_root_role = Roles::extract_privilege_to_capability<LibraRootRole>(account);
        LibraSystem::add_validator(&assoc_root_role, {{alice}});
        Roles::restore_capability_to_privilege(account, assoc_root_role);

        assert(LibraSystem::is_validator({{alice}}), 77);
        assert(LibraSystem::is_validator({{bob}}), 78);
    }
}
// check: NewEpochEvent
// check: EXECUTED
