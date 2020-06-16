script {
    use 0x1::LibraSystem;
    use 0x1::Roles::{Self, AssociationRootRole};

    // Add Validator to the set, called by the validator's operator
    fun add_validator(account: &signer, validator_address: address) {
        let assoc_root_role = Roles::extract_privilege_to_capability<AssociationRootRole>(account);
        LibraSystem::add_validator(&assoc_root_role, validator_address);
        Roles::restore_capability_to_privilege(account, assoc_root_role);
    }
}
