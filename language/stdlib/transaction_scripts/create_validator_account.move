script {
    use 0x1::LibraAccount;
    use 0x1::SlidingNonce;
    /// Create a validator account at `new_validator_address` with `auth_key_prefix`and human_name.
    fun create_validator_account(
        creator: &signer,
        sliding_nonce: u64,
        new_account_address: address,
        auth_key_prefix: vector<u8>,
        human_name: vector<u8>,
        ) {
        SlidingNonce::record_nonce_or_abort(creator, sliding_nonce);
        LibraAccount::create_validator_account(
            creator,
            new_account_address,
            auth_key_prefix,
            human_name,
        );
    }
}
