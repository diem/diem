script {
    use 0x1::LibraSystem;
    use 0x1::Roles::{Self, LibraRootRole};

    /// Update configs of all the validators and emit reconfiguration event.
    fun reconfigure(account: &signer) {
        let assoc_root_role = Roles::extract_privilege_to_capability<LibraRootRole>(account);
        LibraSystem::update_and_reconfigure(&assoc_root_role);
        Roles::restore_capability_to_privilege(account, assoc_root_role);
    }
}
