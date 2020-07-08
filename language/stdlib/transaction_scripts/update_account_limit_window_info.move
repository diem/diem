script {
use 0x1::AccountLimits;
/// * Sets the account limits window `tracking_balance` field for `CoinType` at `window_address` to `aggregate_balance` if `aggregate_balance != 0`.
/// * Sets the account limits window `limit_address` field for `CoinType` at `window_address` to `new_limit_address`.
fun update_account_limit_window_info<CoinType>(
    tc_account: &signer,
    window_address: address,
    aggregate_balance: u64,
    new_limit_address: address
) {
    AccountLimits::update_window_info<CoinType>(tc_account, window_address, aggregate_balance, new_limit_address);
}
}
