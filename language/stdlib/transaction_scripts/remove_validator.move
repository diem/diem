script {
    use 0x1::LibraSystem;
    use 0x1::Roles::{Self, LibraRootRole};

    /// Adding `to_remove` to the set of pending validator removals. Fails if
    /// the `to_remove` address is already in the validator set or already in the pending removals.
    /// Callable by Validator's operator.
    fun remove_validator(account: &signer, validator_address: address) {
        let assoc_root_role = Roles::extract_privilege_to_capability<LibraRootRole>(account);
        LibraSystem::remove_validator(&assoc_root_role, validator_address);
        Roles::restore_capability_to_privilege(account, assoc_root_role);
    }
}
