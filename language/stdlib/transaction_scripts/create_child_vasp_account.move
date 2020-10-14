script {
use 0x1::LibraAccount;

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
/// | `Errors::INVALID_ARGUMENT`  | `LibraAccount::EMALFORMED_AUTHENTICATION_KEY`            | The `auth_key_prefix` was not of length 32.                                              |
/// | `Errors::REQUIRES_ROLE`     | `Roles::EPARENT_VASP`                                    | The sending account wasn't a Parent VASP account.                                        |
/// | `Errors::ALREADY_PUBLISHED` | `Roles::EROLE_ID`                                        | The `child_address` address is already taken.                                            |
/// | `Errors::LIMIT_EXCEEDED`    | `VASP::ETOO_MANY_CHILDREN`                               | The sending account has reached the maximum number of allowed child accounts.            |
/// | `Errors::NOT_PUBLISHED`     | `Libra::ECURRENCY_INFO`                                  | The `CoinType` is not a registered currency on-chain.                                    |
/// | `Errors::INVALID_STATE`     | `LibraAccount::EWITHDRAWAL_CAPABILITY_ALREADY_EXTRACTED` | The withdrawal capability for the sending account has already been extracted.            |
/// | `Errors::NOT_PUBLISHED`     | `LibraAccount::EPAYER_DOESNT_HOLD_CURRENCY`              | The sending account doesn't have a balance in `CoinType`.                                |
/// | `Errors::LIMIT_EXCEEDED`    | `LibraAccount::EINSUFFICIENT_BALANCE`                    | The sending account doesn't have at least `child_initial_balance` of `CoinType` balance. |
/// | `Errors::INVALID_ARGUMENT`  | `LibraAccount::ECANNOT_CREATE_AT_VM_RESERVED`            | The `child_address` is the reserved address 0x0.                                         |
///
/// # Related Scripts
/// * `Script::create_parent_vasp_account`
/// * `Script::add_currency_to_account`
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
    use 0x1::Signer;
    use 0x1::Errors;
    use 0x1::Roles;

    include LibraAccount::TransactionChecks{sender: parent_vasp}; // properties checked by the prologue.
    let parent_addr = Signer::spec_address_of(parent_vasp);
    let parent_cap = LibraAccount::spec_get_withdraw_cap(parent_addr);
    include LibraAccount::CreateChildVASPAccountAbortsIf<CoinType>{
        parent: parent_vasp, new_account_address: child_address};
    aborts_if child_initial_balance > max_u64() with Errors::LIMIT_EXCEEDED;
    include (child_initial_balance > 0) ==>
        LibraAccount::ExtractWithdrawCapAbortsIf{sender_addr: parent_addr};
    include (child_initial_balance > 0) ==>
        LibraAccount::PayFromAbortsIfRestricted<CoinType>{
            cap: parent_cap,
            payee: child_address,
            amount: child_initial_balance,
            metadata: x"",
            metadata_signature: x""
        };
    include LibraAccount::CreateChildVASPAccountEnsures<CoinType>{
        parent_addr: parent_addr,
        child_addr: child_address,
    };
    ensures LibraAccount::balance<CoinType>(child_address) == child_initial_balance;
    ensures LibraAccount::balance<CoinType>(parent_addr)
        == old(LibraAccount::balance<CoinType>(parent_addr)) - child_initial_balance;

    aborts_with [check]
        Errors::REQUIRES_ROLE,
        Errors::ALREADY_PUBLISHED,
        Errors::LIMIT_EXCEEDED,
        Errors::NOT_PUBLISHED,
        Errors::INVALID_STATE,
        Errors::INVALID_ARGUMENT;

    /// **Access Control:**
    /// Only Parent VASP accounts can create Child VASP accounts [[A7]][ROLE].
    include Roles::AbortsIfNotParentVasp{account: parent_vasp};
}
}
