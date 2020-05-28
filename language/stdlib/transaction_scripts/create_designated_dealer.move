script {
    use 0x0::LibraAccount;
        /// Script for Treasury Compliance Account to create designated dealer account at 'new_account_address'
        /// and 'auth_key_prefix' for nonsynthetic CoinType
        fun main<CoinType>(tc_account: &signer, new_account_address: address, auth_key_prefix: vector<u8>) {
            LibraAccount::create_designated_dealer<CoinType>(tc_account, new_account_address, auth_key_prefix);
        }
    }
