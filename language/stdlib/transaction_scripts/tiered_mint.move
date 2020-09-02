script {
use 0x1::LibraAccount;
use 0x1::SlidingNonce;

/// Mint `mint_amount` to `designated_dealer_address` for `tier_index` tier.
/// Max valid tier index is 3 since there are max 4 tiers per DD.
/// Sender should be treasury compliance account and receiver authorized DD.
/// `sliding_nonce` is a unique nonce for operation, see sliding_nonce.move for details.
fun tiered_mint<CoinType>(
    tc_account: &signer,
    sliding_nonce: u64,
    designated_dealer_address: address,
    mint_amount: u64,
    tier_index: u64
) {
    SlidingNonce::record_nonce_or_abort(tc_account, sliding_nonce);
    LibraAccount::tiered_mint<CoinType>(
        tc_account, designated_dealer_address, mint_amount, tier_index
    );
}
}
