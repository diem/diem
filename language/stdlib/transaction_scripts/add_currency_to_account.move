script {
use 0x1::LibraAccount;
// Prover deps:
use 0x1::Libra;
use 0x1::Roles;
use 0x1::Signer;
use 0x1::VASP;

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
    // TODO(shb): lots of spurious abort conditions that I couldn't figure out
    pragma aborts_if_is_partial = true;

    /// This publishes a `Balance<Currency>` to the caller's account
    ensures exists<LibraAccount::Balance<Currency>>(Signer::spec_address_of(account));

    /// `Currency` must be valid
    aborts_if !Libra::spec_is_currency<Currency>();
    /// `account` must be allowed to hold balances
    aborts_if !Roles::spec_can_hold_balance_addr(Signer::spec_address_of(account));
    /// `account` cannot have an existing balance in `Currency`
    aborts_if exists<LibraAccount::Balance<Currency>>(Signer::spec_address_of(account));
    /// If `account` is a child VASP, its parent must have a published `AccountLimits<Currency>`
    aborts_if
        Roles::spec_needs_account_limits_addr(Signer::spec_address_of(account)) &&
        Roles::spec_has_child_VASP_role_addr(Signer::spec_address_of(account)) &&
        !VASP::spec_has_account_limits<Currency>(Signer::spec_address_of(account)) &&
        // TODO(shb): teach prover that Roles::spec_has_child_VASP_roles ==> VASP::spec_is_vasp and
        // then eliminate this
        VASP::spec_is_vasp(Signer::spec_address_of(account));
}
}
