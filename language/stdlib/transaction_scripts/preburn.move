script {
use 0x0::Libra;
use 0x0::LibraAccount;

/// Preburn `amount` `Token`s from `account`.
/// This will only succeed if `account` already has a published `Preburn<Token>` resource.
fun main<Token>(account: &signer, amount: u64) {
    Libra::preburn_to<Token>(account, LibraAccount::withdraw_from(account, amount))
}
}
