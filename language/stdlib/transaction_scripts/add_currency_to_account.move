script {
use 0x1::LibraAccount;
// Prover deps:
use 0x1::Libra;
use 0x1::Roles;
use 0x1::Signer;

/// Add a `Currency` balance to `account`, which will enable `account` to send and receive
/// `Libra<Currency>`.
/// Aborts with NOT_A_CURRENCY if `Currency` is not an accepted currency type in the Libra system
/// Aborts with `LibraAccount::ADD_EXISTING_CURRENCY` if the account already holds a balance in
/// `Currency`.
fun add_currency_to_account<Currency>(account: &signer) {
    LibraAccount::add_currency<Currency>(account);
}
spec fun add_currency_to_account {
    pragma verify = true;

    /// This publishes a `Balance<Currency>` to the caller's account
    ensures exists<LibraAccount::Balance<Currency>>(Signer::spec_address_of(account));

    /// `Currency` must be valid
    aborts_if !Libra::spec_is_currency<Currency>();
    /// `account` must be allowed to hold balances
    aborts_if !Roles::spec_can_hold_balance_addr(Signer::spec_address_of(account));
    /// `account` cannot have an existing balance in `Currency`
    aborts_if exists<LibraAccount::Balance<Currency>>(Signer::spec_address_of(account));
}
}
