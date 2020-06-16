script {
    use 0x1::LibraAccount;

    fun create_validator_account<Token>(creator: &signer, new_account_address: address, auth_key_prefix: vector<u8>) {
        LibraAccount::create_validator_account<Token>(
            creator,
            new_account_address,
            auth_key_prefix
        );
    }
}
