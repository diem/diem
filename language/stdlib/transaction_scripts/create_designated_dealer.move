script {
    use 0x1::DesignatedDealer;
    use 0x1::LibraAccount;
    use 0x1::SlidingNonce;

        /// Create designated dealer account at 'new_account_address' and 'auth_key_prefix' for nonsynthetic CoinType.
        /// Create dealer and preburn resource.
        fun create_designated_dealer<CoinType>(tc_account: &signer, sliding_nonce: u64, new_account_address: address, auth_key_prefix: vector<u8>) {
            // XXX We need to figure out if TC is in charge of this or association root account. For now we assume assoc root.
            SlidingNonce::record_nonce_or_abort(tc_account, sliding_nonce);
            LibraAccount::create_designated_dealer<CoinType>(
                tc_account,
                new_account_address,
                auth_key_prefix
            );
            // Create default tiers for newly created DD
            DesignatedDealer::add_tier(tc_account, new_account_address, 500000);
            DesignatedDealer::add_tier(tc_account, new_account_address, 5000000);
            DesignatedDealer::add_tier(tc_account, new_account_address, 50000000);
            DesignatedDealer::add_tier(tc_account, new_account_address, 500000000);
        }
    }
