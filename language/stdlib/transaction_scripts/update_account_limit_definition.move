script {
use 0x1::AccountLimits;
use 0x1::SlidingNonce;

/// Optionally update thresholds of max balance, inflow, outflow
/// for any limits-bound accounts with their limits defined at `limit_address`.
/// Limits are defined in terms of base (on-chain) currency units for `CoinType`.
/// If a new threshold is 0, that particular config does not get updated.
/// `sliding_nonce` is a unique nonce for operation, see SlidingNonce.move for details.
fun update_account_limit_definition<CoinType>(
    tc_account: &signer,
    limit_address: address,
    sliding_nonce: u64,
    new_max_inflow: u64,
    new_max_outflow: u64,
    new_max_holding_balance: u64,
    new_time_period: u64,
) {
    SlidingNonce::record_nonce_or_abort(tc_account, sliding_nonce);
    AccountLimits::update_limits_definition<CoinType>(
        tc_account,
        limit_address,
        new_max_inflow,
        new_max_outflow,
        new_max_holding_balance,
        new_time_period,
    );
}
}
