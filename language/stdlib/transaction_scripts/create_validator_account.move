script {
    use 0x1::LibraAccount;
    use 0x1::Roles::{Self, AssociationRootRole};

    fun create_validator_account(creator: &signer, new_account_address: address, auth_key_prefix: vector<u8>) {
        let assoc_root_role = Roles::extract_privilege_to_capability<AssociationRootRole>(creator);
        LibraAccount::create_validator_account(
            creator,
            &assoc_root_role,
            new_account_address,
            auth_key_prefix
        );
        Roles::restore_capability_to_privilege(creator, assoc_root_role);
    }
}
