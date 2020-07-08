script {
use 0x1::AccountLimits;

/// Publishes an unrestricted `LimitsDefintion<CoinType>` under `account`.
/// Will abort if a resource with the same type already exists under `account`.
/// No windows will point to this limit at the time it is published.
fun publish_account_limit_definition<CoinType>(account: &signer) {
    AccountLimits::publish_unrestricted_limits<CoinType>(account);
}
}
