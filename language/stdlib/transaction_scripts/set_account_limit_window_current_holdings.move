script {
use 0x1::AccountLimits;
/// Sets the account limits window `tracking_balance` field for `CointType` at `window_address` to `aggregate_balance`
fun set_account_limit_window_current_holdings<CointType>(tc_account: &signer,  window_address: address, aggregate_balance: u64) {
    AccountLimits::set_current_holdings<CointType>(tc_account, window_address, aggregate_balance);
}
}
