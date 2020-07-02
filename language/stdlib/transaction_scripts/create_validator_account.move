script {
    use 0x1::LibraAccount;

    /// Create a validator account at `new_validator_address` with `auth_key_prefix`.
    fun create_validator_account(
        creator: &signer,
        new_account_address: address,
        auth_key_prefix: vector<u8>,
        ) {
        LibraAccount::create_validator_account(
            creator,
            new_account_address,
            auth_key_prefix
        );
    }
}
