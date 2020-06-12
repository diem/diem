script {
use 0x1::AccountLimits;
use 0x1::SlidingNonce;

/// Script for Treasury Comliance Account to optionally update global thresholds
/// of max balance, total flow (inflow + outflow) (microLBR) for LimitsDefinition bound accounts.
/// If the new threshold is zero, that particular config does not get updated.
/// sliding_nonce is a unique nonce for operation, see sliding_nonce.move for details
fun main<CoinType>(
    tc_account: &signer,
    sliding_nonce: u64,
    new_max_total_flow: u64,
    new_max_holding_balance: u64,
) {
    SlidingNonce::record_nonce_or_abort(tc_account, sliding_nonce);
    AccountLimits::update_limits_definition(tc_account, new_max_total_flow, new_max_holding_balance);
}
}
