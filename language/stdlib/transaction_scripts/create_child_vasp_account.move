script {
use 0x1::LibraAccount;

// imports for the prover
use 0x1::VASP;
use 0x1::Roles;
use 0x1::Signer;

/// Create a `ChildVASP` account for sender `parent_vasp` at `child_address` with a balance of
/// `child_initial_balance` in `CoinType` and an initial authentication_key
/// `auth_key_prefix | child_address`.
/// If `add_all_currencies` is true, the child address will have a zero balance in all available
/// currencies in the system.
/// This account will a child of the transaction sender, which must be a ParentVASP.
///
/// ## Aborts
/// The transaction will abort:
///
/// * If `parent_vasp` is not a parent vasp with error: `Roles::EINVALID_PARENT_ROLE`
/// * If `child_address` already exists with error: `Roles::EROLE_ALREADY_ASSIGNED`
/// * If `parent_vasp` already has 256 child accounts with error: `VASP::ETOO_MANY_CHILDREN`
/// * If `CoinType` is not a registered currency with error: `LibraAccount::ENOT_A_CURRENCY`
/// * If `parent_vasp`'s withdrawal capability has been extracted with error:  `LibraAccount::EWITHDRAWAL_CAPABILITY_ALREADY_EXTRACTED`
/// * If `parent_vasp` doesn't hold `CoinType` and `child_initial_balance > 0` with error: `LibraAccount::EPAYER_DOESNT_HOLD_CURRENCY`
/// * If `parent_vasp` doesn't at least `child_initial_balance` of `CoinType` in its account balance with error: `LibraAccount::EINSUFFICIENT_BALANCE`
fun create_child_vasp_account<CoinType>(
    parent_vasp: &signer,
    child_address: address,
    auth_key_prefix: vector<u8>,
    add_all_currencies: bool,
    child_initial_balance: u64
) {
    LibraAccount::create_child_vasp_account<CoinType>(
        parent_vasp,
        child_address,
        auth_key_prefix,
        add_all_currencies,
    );
    // Give the newly created child `child_initial_balance` coins
    if (child_initial_balance > 0) {
        let vasp_withdrawal_cap = LibraAccount::extract_withdraw_capability(parent_vasp);
        LibraAccount::pay_from<CoinType>(
            &vasp_withdrawal_cap, child_address, child_initial_balance, x"", x""
        );
        LibraAccount::restore_withdraw_capability(vasp_withdrawal_cap);
    };
}
spec fun create_child_vasp_account {
    pragma verify = false;
    pragma aborts_if_is_partial = true;
    /// `parent_vasp` must be a parent vasp account
    // TODO(tzakian): need to teach the prover that Roles::has_parent_VASP_role ==> VASP::spec_is_parent_vasp
    aborts_if !Roles::spec_has_parent_VASP_role_addr(Signer::spec_address_of(parent_vasp));
    aborts_if !VASP::spec_is_parent_vasp(Signer::spec_address_of(parent_vasp));
    /// `child_address` must not be an existing account/vasp account
    // TODO(tzakian): need to teach the prover that !exists(account) ==> !VASP::spec_is_vasp(child_address)
    aborts_if exists<LibraAccount::LibraAccount>(child_address);
    aborts_if VASP::spec_is_vasp(child_address);
    /// `parent_vasp` must not have created more than 256 children
    aborts_if VASP::spec_get_num_children(Signer::spec_address_of(parent_vasp)) + 1 > 256; // MAX_CHILD_ACCOUNTS
}
}
