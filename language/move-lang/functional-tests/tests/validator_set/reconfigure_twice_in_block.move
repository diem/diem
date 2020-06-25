// Checks that only one change to the validator set can be made per block:
// if the first transaction removes a validator and the second transaction
// removes a validator in the same block, then the first transaction will succeed
// and the second will fail.

//! account: alice, 1000000, 0, validator
//! account: bob, 1000000, 0, validator
//! account: carrol, 1000000, 0, validator

//! block-prologue
//! proposer: bob
//! block-time: 2

// check: EXECUTED

//! new-transaction
//! sender: association
script {
    use 0x1::LibraSystem;
    use 0x1::Roles::{Self, LibraRootRole};
    fun main(account: &signer) {
        let assoc_root_role = Roles::extract_privilege_to_capability<LibraRootRole>(account);
        LibraSystem::remove_validator(&assoc_root_role, {{alice}});
        Roles::restore_capability_to_privilege(account, assoc_root_role);
    }
}

// check: NewEpochEvent
// check: EXECUTED

//! new-transaction
//! sender: association
script {
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
//! block-time: 3

// check: EXECUTED

//! new-transaction
//! sender: association
script {
    use 0x1::LibraSystem;
    use 0x1::Roles::{Self, LibraRootRole};
    fun main(account: &signer) {
        let assoc_root_role = Roles::extract_privilege_to_capability<LibraRootRole>(account);
        LibraSystem::remove_validator(&assoc_root_role, {{bob}});
        Roles::restore_capability_to_privilege(account, assoc_root_role);
    }
}

// check: NewEpochEvent
// check: EXECUTED
