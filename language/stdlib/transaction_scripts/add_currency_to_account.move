script {
use 0x1::LibraAccount;

/// Add the currency identified by the type `currency` to the sending accounts.
/// Aborts if the account already holds a balance fo `currency` type.
fun add_currency_to_account<Currency>(account: &signer) {
    LibraAccount::add_currency<Currency>(account);
}
}
