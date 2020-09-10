script {
use 0x1::LibraAccount;

/// Preburn `amount` `Token`s from `account`.
/// This will only succeed if `account` already has a published `Preburn<Token>` resource.
fun preburn<Token>(account: &signer, amount: u64) {
    let withdraw_cap = LibraAccount::extract_withdraw_capability(account);
    LibraAccount::preburn<Token>(account, &withdraw_cap, amount);
    LibraAccount::restore_withdraw_capability(withdraw_cap);
}
}
