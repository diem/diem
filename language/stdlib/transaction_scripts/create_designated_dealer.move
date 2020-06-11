script {
    use 0x1::LibraAccount;
    use 0x1::SlidingNonce;
        /// Script for Treasury Compliance Account to create designated dealer account at 'new_account_address'
        /// and 'auth_key_prefix' for nonsynthetic CoinType. Creates dealer and preburn resource for dd.
        fun main<CoinType>(tc_account: &signer, sliding_nonce: u64, new_account_address: address, auth_key_prefix: vector<u8>) {
            SlidingNonce::record_nonce_or_abort(tc_account, sliding_nonce);
            LibraAccount::create_designated_dealer<CoinType>(tc_account, new_account_address, auth_key_prefix)
        }
    }
