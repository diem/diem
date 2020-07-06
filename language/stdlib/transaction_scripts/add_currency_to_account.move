script {
use 0x1::LibraAccount;
// Prover deps:
use 0x1::Libra;
use 0x1::Signer;

/// Add a `Currency` balance to `account`, which will enable `account` to send and receive
/// `Libra<Currency>`.
/// Aborts with NOT_A_CURRENCY if `Currency` is not an accepted currency type in the Libra system
/// Aborts with `LibraAccount::ADD_EXISTING_CURRENCY` if the account already holds a balance in
/// `Currency`.
/// Aborts with `LibraAccount::PARENT_VASP_CURRENCY_LIMITS_DNE` if `account` is a `ChildVASP` whose
/// parent does not have an `AccountLimits<Currency>` resource.
fun add_currency_to_account<Currency>(account: &signer) {
    LibraAccount::add_currency<Currency>(account);
}
spec fun add_currency_to_account {
    pragma verify = true;
    pragma aborts_if_is_partial = true;
    // `Currency` must be valid
    aborts_if !Libra::spec_is_currency<Currency>();
    // Can't have existing balance in `Currency`
    aborts_if exists<LibraAccount::Balance<Currency>>(Signer::spec_address_of(account));
    // TODO(shb): Prover claims that there's no abort in this situation. I suspect we need to
    // rewrite VASP::try_allow_currency to be more amenable to verification
    // If child VASP, parent needs `AccountLimits<Currency>`
    //aborts_if VASP::spec_is_child_vasp(Signer::spec_address_of(account)) && !exists<AccountLimits::Window<Currency>>(VASP::spec_parent_address(Signer::spec_address_of(account)));
}
}
