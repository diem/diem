script {
use 0x1::DesignatedDealer;
use 0x1::LibraAccount;
use 0x1::SlidingNonce;

/// Script for Treasury Comliance Account to mint 'mint_amount' to 'designated_dealer_address' for
/// 'tier_index' tier
/// sliding_nonce is a unique nonce for operation, see sliding_nonce.move for details
fun main<CoinType>(
    tc_account: &signer,
    sliding_nonce: u64,
    designated_dealer_address: address,
    mint_amount: u64,
    tier_index: u64
) {
    SlidingNonce::record_nonce_or_abort(tc_account, sliding_nonce);
    let coins = DesignatedDealer::tiered_mint<CoinType>(
        tc_account, mint_amount, designated_dealer_address, tier_index
    );
    LibraAccount::deposit(tc_account, designated_dealer_address, coins)
}
}
