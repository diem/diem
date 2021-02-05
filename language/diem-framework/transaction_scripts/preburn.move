script {
use 0x1::DiemAccount;

/// # Summary
/// Moves a specified number of coins in a given currency from the account's
/// balance to its preburn area after which the coins may be burned. This
/// transaction may be sent by any account that holds a balance and preburn area
/// in the specified currency.
///
/// # Technical Description
/// Moves the specified `amount` of coins in `Token` currency from the sending `account`'s
/// `DiemAccount::Balance<Token>` to the `Diem::Preburn<Token>` published under the same
/// `account`. `account` must have both of these resources published under it at the start of this
/// transaction in order for it to execute successfully.
///
/// ## Events
/// Successful execution of this script emits two events:
/// * `DiemAccount::SentPaymentEvent ` on `account`'s `DiemAccount::DiemAccount` `sent_events`
/// handle with the `payee` and `payer` fields being `account`'s address; and
/// * A `Diem::PreburnEvent` with `Token`'s currency code on the
/// `Diem::CurrencyInfo<Token`'s `preburn_events` handle for `Token` and with
/// `preburn_address` set to `account`'s address.
///
/// # Parameters
/// | Name      | Type      | Description                                                                                                                      |
/// | ------    | ------    | -------------                                                                                                                    |
/// | `Token`   | Type      | The Move type for the `Token` currency being moved to the preburn area. `Token` must be an already-registered currency on-chain. |
/// | `account` | `&signer` | The signer reference of the sending account.                                                                                     |
/// | `amount`  | `u64`     | The amount in `Token` to be moved to the preburn area.                                                                           |
///
/// # Common Abort Conditions
/// | Error Category           | Error Reason                                             | Description                                                                             |
/// | ----------------         | --------------                                           | -------------                                                                           |
/// | `Errors::NOT_PUBLISHED`  | `Diem::ECURRENCY_INFO`                                  | The `Token` is not a registered currency on-chain.                                      |
/// | `Errors::INVALID_STATE`  | `DiemAccount::EWITHDRAWAL_CAPABILITY_ALREADY_EXTRACTED` | The withdrawal capability for `account` has already been extracted.                     |
/// | `Errors::LIMIT_EXCEEDED` | `DiemAccount::EINSUFFICIENT_BALANCE`                    | `amount` is greater than `payer`'s balance in `Token`.                                  |
/// | `Errors::NOT_PUBLISHED`  | `DiemAccount::EPAYER_DOESNT_HOLD_CURRENCY`              | `account` doesn't hold a balance in `Token`.                                            |
/// | `Errors::NOT_PUBLISHED`  | `Diem::EPREBURN`                                        | `account` doesn't have a `Diem::Preburn<Token>` resource published under it.           |
/// | `Errors::INVALID_STATE`  | `Diem::EPREBURN_OCCUPIED`                               | The `value` field in the `Diem::Preburn<Token>` resource under the sender is non-zero. |
/// | `Errors::NOT_PUBLISHED`  | `Roles::EROLE_ID`                                        | The `account` did not have a role assigned to it.                                       |
/// | `Errors::REQUIRES_ROLE`  | `Roles::EDESIGNATED_DEALER`                              | The `account` did not have the role of DesignatedDealer.                                |
///
/// # Related Scripts
/// * `Script::cancel_burn`
/// * `Script::burn`
/// * `Script::burn_txn_fees`

fun preburn<Token>(account: &signer, amount: u64) {
    let withdraw_cap = DiemAccount::extract_withdraw_capability(account);
    DiemAccount::preburn<Token>(account, &withdraw_cap, amount);
    DiemAccount::restore_withdraw_capability(withdraw_cap);
}

spec fun preburn {
    use 0x1::Errors;
    use 0x1::Signer;
    use 0x1::Diem;

    include DiemAccount::TransactionChecks{sender: account}; // properties checked by the prologue.
    let account_addr = Signer::spec_address_of(account);
    let cap = DiemAccount::spec_get_withdraw_cap(account_addr);
    include DiemAccount::ExtractWithdrawCapAbortsIf{sender_addr: account_addr};
    include DiemAccount::PreburnAbortsIf<Token>{dd: account, cap: cap};
    include DiemAccount::PreburnEnsures<Token>{dd_addr: account_addr, payer: account_addr};

    aborts_with [check]
        Errors::NOT_PUBLISHED,
        Errors::INVALID_STATE,
        Errors::REQUIRES_ROLE,
        Errors::LIMIT_EXCEEDED;

    /// **Access Control:**
    /// Only the account with a preburn area can preburn [[H4]][PERMISSION].
    include Diem::AbortsIfNoPreburn<Token>{preburn_address: account_addr};
}
}
