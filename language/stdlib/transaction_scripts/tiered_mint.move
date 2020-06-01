script {
use 0x0::LibraAccount;
use 0x0::SlidingNonce;

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
    LibraAccount::mint_to_designated_dealer<CoinType>(
        tc_account, designated_dealer_address, mint_amount, tier_index
    );
}
}
