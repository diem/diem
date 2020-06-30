//! account: alice
//! account: vivian, 1000000, 0, validator
//! account: viola, 1000000, 0, validator
//! account: v1, 1000000, 0, validator
//! account: v2, 1000000, 0, validator
//! account: v3, 1000000, 0, validator

//! block-prologue
//! proposer: viola
//! block-time: 2

//! new-transaction
//! sender: association
script{
    use 0x1::LibraSystem;
    use 0x1::Roles::{Self, LibraRootRole};
    fun main(account: &signer) {
        let assoc_root_role = Roles::extract_privilege_to_capability<LibraRootRole>(account);
        LibraSystem::remove_validator(&assoc_root_role, {{vivian}});
        Roles::restore_capability_to_privilege(account, assoc_root_role);
    }
}

// check: NewEpochEvent
// check: EXECUTED

//! block-prologue
//! proposer: viola
//! block-time: 4

//! new-transaction
// check that Vivian is no longer a validator, Alice is not, but Viola is still a
// validator
script{
    use 0x1::LibraSystem;
    fun main() {
        assert(!LibraSystem::is_validator({{vivian}}), 70);
        assert(!LibraSystem::is_validator({{alice}}), 71);
        assert(LibraSystem::is_validator({{viola}}), 72);
    }
}

// check: EXECUTED
