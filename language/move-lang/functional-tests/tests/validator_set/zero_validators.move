//! account: vivian, 1000000, 0, validator

//! block-prologue
//! proposer: vivian
//! block-time: 3

//! new-transaction
//! sender: association
script {
    use 0x1::LibraSystem;
    fun main() {
        LibraSystem::get_validator_config({{vivian}});
    }
}
// check: EXECUTED

//! new-transaction
//! sender: association
script {
    use 0x1::LibraSystem;
    use 0x1::Roles::{Self, AssociationRootRole};
    fun main(account: &signer) {
        let assoc_root_role = Roles::extract_privilege_to_capability<AssociationRootRole>(account);
        let num_validators = LibraSystem::validator_set_size();
        assert(num_validators == 1, 98);
        let index = 0;
        while (index < num_validators) {
            let addr = LibraSystem::get_ith_validator_address(index);
            LibraSystem::remove_validator(&assoc_root_role, addr);
            index = index + 1;
        };
        Roles::restore_capability_to_privilege(account, assoc_root_role);
    }
}
// check: EXECUTED

//! new-transaction
//! sender: association
script {
    use 0x1::LibraSystem;
    use 0x1::Roles::{Self, AssociationRootRole};
    fun main(account: &signer) {
        let assoc_root_role = Roles::extract_privilege_to_capability<AssociationRootRole>(account);
        LibraSystem::update_and_reconfigure(&assoc_root_role);
        Roles::restore_capability_to_privilege(account, assoc_root_role);
        let num_validators = LibraSystem::validator_set_size();
        assert(num_validators == 0, 98);
    }
}
// check: EXECUTED

//! new-transaction
//! sender: association
script {
    use 0x1::LibraSystem;
    fun main() {
        LibraSystem::get_validator_config({{vivian}});
    }
}
// check: ABORTED
// check: 33
