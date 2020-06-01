script {
use 0x0::Libra;

// Publish a newly created `Preburn<Token>` resource under `account`.
// This will abort if `account` already has a published `Preburn<Token>` resource.
fun main<Token>(account: &signer) {
    Libra::publish_preburn(account, Libra::new_preburn<Token>())
}
}
