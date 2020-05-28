script {
    use 0x0::LibraAccount;
        /// Script for Treasury Comliance Account to mint 'mint_amount' to 'designated_dealer_address'
        /// for 'tier_index' tier
        fun main<CoinType>(tc_account: &signer, designated_dealer_address: address, mint_amount: u64, tier_index: u64) {
            LibraAccount::mint_to_designated_dealer<CoinType>(tc_account, designated_dealer_address, mint_amount, tier_index);
        }
    }
