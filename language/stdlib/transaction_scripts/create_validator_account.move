script {
    use 0x1::LibraAccount;
    use 0x1::RecoveryAddress;

    /// Create a validator account at `new_validator_address` with `auth_key_prefix`.
    fun create_validator_account(
        creator: &signer,
        new_account_address: address,
        auth_key_prefix: vector<u8>,
        ) {
        let rotation_cap = LibraAccount::create_validator_account_and_extract_key_rotation_cap(
            creator,
            new_account_address,
            auth_key_prefix
        );
        // LibraAccount::restore_key_rotation_capability(rotation_cap);
        RecoveryAddress::add_rotation_capability_explicitly(creator, rotation_cap);
        
    }
}
