script {
    use 0x0::LibraAccount;

    fun main<Token>(creator: &signer, new_account_address: address, auth_key_prefix: vector<u8>) {
        LibraAccount::create_validator_account<Token>(
            creator,
            new_account_address,
            auth_key_prefix
        );
    }
}
