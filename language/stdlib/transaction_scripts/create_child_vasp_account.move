script {
use 0x1::LibraAccount;

// imports for the prover
use 0x1::VASP;
use 0x1::Roles;
use 0x1::Signer;

/// # Summary
/// Creates a Child VASP account with its parent being the sending account of the transaction.
/// The sender of the transaction must be a Parent VASP account.
///
/// # Technical Description
/// Creates a `ChildVASP` account for the sender `parent_vasp` at `child_address` with a balance of
/// `child_initial_balance` in `CoinType` and an initial authentication key of
/// `auth_key_prefix | child_address`.
///
/// If `add_all_currencies` is true, the child address will have a zero balance in all available
/// currencies in the system.
///
/// The new account will be a child account of the transaction sender, which must be a
/// Parent VASP account. The child account will be recorded against the limit of
/// child accounts of the creating Parent VASP account.
///
/// ## Events
/// Successful execution with a `child_initial_balance` greater than zero will emit:
/// * A `LibraAccount::SentPaymentEvent` with the `payer` field being the Parent VASP's address,
/// and payee field being `child_address`. This is emitted on the Parent VASP's
/// `LibraAccount::LibraAccount` `sent_events` handle.
/// * A `LibraAccount::ReceivedPaymentEvent` with the  `payer` field being the Parent VASP's address,
/// and payee field being `child_address`. This is emitted on the new Child VASPS's
/// `LibraAccount::LibraAccount` `received_events` handle.
///
/// # Parameters
/// | Name                    | Type         | Description                                                                                                                                 |
/// | ------                  | ------       | -------------                                                                                                                               |
/// | `CoinType`              | Type         | The Move type for the `CoinType` that the child account should be created with. `CoinType` must be an already-registered currency on-chain. |
/// | `parent_vasp`           | `&signer`    | The signer reference of the sending account. Must be a Parent VASP account.                                                                 |
/// | `child_address`         | `address`    | Address of the to-be-created Child VASP account.                                                                                            |
/// | `auth_key_prefix`       | `vector<u8>` | The authentication key prefix that will be used initially for the newly created account.                                                    |
/// | `add_all_currencies`    | `bool`       | Whether to publish balance resources for all known currencies when the account is created.                                                  |
/// | `child_initial_balance` | `u64`        | The initial balance in `CoinType` to give the child account when it's created.                                                              |
///
/// # Common Abort Conditions
/// | Error Category              | Error Reason                                             | Description                                                                              |
/// | ----------------            | --------------                                           | -------------                                                                            |
/// | `Errors::REQUIRES_ROLE`     | `Roles::EPARENT_VASP`                                    | The sending account wasn't a Parent VASP account.                                        |
/// | `Errors::ALREADY_PUBLISHED` | `Roles::EROLE_ID`                                        | The `child_address` address is already taken.                                            |
/// | `Errors::LIMIT_EXCEEDED`    | `VASP::ETOO_MANY_CHILDREN`                               | The sending account has reached the maximum number of allowed child accounts.            |
/// | `Errors::NOT_PUBLISHED`     | `Libra::ECURRENCY_INFO`                                  | The `CoinType` is not a registered currency on-chain.                                    |
/// | `Errors::INVALID_STATE`     | `LibraAccount::EWITHDRAWAL_CAPABILITY_ALREADY_EXTRACTED` | The withdrawal capability for the sending account has already been extracted.            |
/// | `Errors::NOT_PUBLISHED`     | `LibraAccount::EPAYER_DOESNT_HOLD_CURRENCY`              | The sending account doesn't have a balance in `CoinType`.                                |
/// | `Errors::LIMIT_EXCEEDED`    | `LibraAccount::EINSUFFICIENT_BALANCE`                    | The sending account doesn't have at least `child_initial_balance` of `CoinType` balance. |
///
/// # Related Scripts
/// * `Script::create_parent_vasp_account`
/// * `Script::add_currency`
/// * `Script::rotate_authentication_key`
/// * `Script::add_recovery_rotation_capability`
/// * `Script::create_recovery_address`

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
    // TODO(tzakian): need to teach the prover that Roles::has_parent_VASP_role ==> VASP::is_parent
    aborts_if !Roles::spec_has_parent_VASP_role_addr(Signer::spec_address_of(parent_vasp));
    aborts_if !VASP::is_parent(Signer::spec_address_of(parent_vasp));
    /// `child_address` must not be an existing account/vasp account
    // TODO(tzakian): need to teach the prover that !exists(account) ==> !VASP::is_vasp(child_address)
    aborts_if exists<LibraAccount::LibraAccount>(child_address);
    aborts_if VASP::is_vasp(child_address);
    /// `parent_vasp` must not have created more than 256 children
    aborts_if VASP::spec_get_num_children(Signer::spec_address_of(parent_vasp)) + 1 > 256; // MAX_CHILD_ACCOUNTS
}
}
