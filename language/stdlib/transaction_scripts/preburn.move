script {
use 0x1::LibraAccount;

/// # Summary
/// Moves a specified number of coins in a given currency from the account's
/// balance to its preburn area after which the coins may be burned. This
/// transaction may be sent by any account that holds a balance and preburn area
/// in the specified currency.
///
/// # Technical Description
/// Moves the specified `amount` of coins in `Token` currency from the sending `account`'s
/// `LibraAccount::Balance<Token>` to the `Libra::Preburn<Token>` published under the same
/// `account`. `account` must have both of these resources published under it at the start of this
/// transaction in order for it to execute successfully.
///
/// ## Events
/// Successful execution of this script emits two events:
/// * `LibraAccount::SentPaymentEvent ` on `account`'s `LibraAccount::LibraAccount` `sent_events`
/// handle with the `payee` and `payer` fields being `account`'s address; and
/// * A `Libra::PreburnEvent` with `Token`'s currency code on the
/// `Libra::CurrencyInfo<Token`'s `preburn_events` handle for `Token` and with
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
/// | `Errors::NOT_PUBLISHED`  | `Libra::ECURRENCY_INFO`                                  | The `Token` is not a registered currency on-chain.                                      |
/// | `Errors::INVALID_STATE`  | `LibraAccount::EWITHDRAWAL_CAPABILITY_ALREADY_EXTRACTED` | The withdrawal capability for `account` has already been extracted.                     |
/// | `Errors::LIMIT_EXCEEDED` | `LibraAccount::EINSUFFICIENT_BALANCE`                    | `amount` is greater than `payer`'s balance in `Token`.                                  |
/// | `Errors::NOT_PUBLISHED`  | `LibraAccount::EPAYER_DOESNT_HOLD_CURRENCY`              | `account` doesn't hold a balance in `Token`.                                            |
/// | `Errors::NOT_PUBLISHED`  | `Libra::EPREBURN`                                        | `account` doesn't have a `Libra::Preburn<Token>` resource published under it.           |
/// | `Errors::INVALID_STATE`  | `Libra::EPREBURN_OCCUPIED`                               | The `value` field in the `Libra::Preburn<Token>` resource under the sender is non-zero. |
///
/// # Related Scripts
/// * `Script::cancel_burn`
/// * `Script::burn`
/// * `Script::burn_txn_fees`

fun preburn<Token>(account: &signer, amount: u64) {
    let withdraw_cap = LibraAccount::extract_withdraw_capability(account);
    LibraAccount::preburn<Token>(account, &withdraw_cap, amount);
    LibraAccount::restore_withdraw_capability(withdraw_cap);
}
}
