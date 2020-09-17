script {
use 0x1::LibraAccount;

/// # Summary
/// Cancels and returns all coins held in the preburn area under
/// `preburn_address` and returns the funds to the `preburn_address`'s balance.
/// Can only be successfully sent by an account with Treasury Compliance role.
///
/// # Technical Description
/// Cancels and returns all coins held in the `Libra::Preburn<Token>` resource under the `preburn_address` and
/// return the funds to the `preburn_address` account's `LibraAccount::Balance<Token>`.
/// The transaction must be sent by an `account` with a `Libra::BurnCapability<Token>`
/// resource published under it. The account at `preburn_address` must have a
/// `Libra::Preburn<Token>` resource published under it, and its value must be nonzero. The transaction removes
/// the entire balance held in the `Libra::Preburn<Token>` resource, and returns it back to the account's
/// `LibraAccount::Balance<Token>` under `preburn_address`. Due to this, the account at
/// `preburn_address` must already have a balance in the `Token` currency published
/// before this script is called otherwise the transaction will fail.
///
/// ## Events
/// The successful execution of this transaction will emit:
/// * A `Libra::CancelBurnEvent` on the event handle held in the `Libra::CurrencyInfo<Token>`
/// resource's `burn_events` published under `0xA550C18`.
/// * A `LibraAccount::ReceivedPaymentEvent` on the `preburn_address`'s
/// `LibraAccount::LibraAccount` `received_events` event handle with both the `payer` and `payee`
/// being `preburn_address`.
///
/// # Parameters
/// | Name              | Type      | Description                                                                                                                          |
/// | ------            | ------    | -------------                                                                                                                        |
/// | `Token`           | Type      | The Move type for the `Token` currenty that burning is being cancelled for. `Token` must be an already-registered currency on-chain. |
/// | `account`         | `&signer` | The signer reference of the sending account of this transaction, must have a burn capability for `Token` published under it.         |
/// | `preburn_address` | `address` | The address where the coins to-be-burned are currently held.                                                                         |
///
/// # Common Abort Conditions
/// | Error Category                | Error Reason                                     | Description                                                                                           |
/// | ----------------              | --------------                                   | -------------                                                                                         |
/// | `Errors::REQUIRES_CAPABILITY` | `Libra::EBURN_CAPABILITY`                        | The sending `account` does not have a `Libra::BurnCapability<Token>` published under it.              |
/// | `Errors::NOT_PUBLISHED`       | `Libra::EPREBURN`                                | The account at `preburn_address` does not have a `Libra::Preburn<Token>` resource published under it. |
/// | `Errors::NOT_PUBLISHED`       | `Libra::ECURRENCY_INFO`                          | The specified `Token` is not a registered currency on-chain.                                          |
/// | `Errors::INVALID_ARGUMENT`    | `LibraAccount::ECOIN_DEPOSIT_IS_ZERO`            | The value held in the preburn resource was zero.                                                      |
/// | `Errors::INVALID_ARGUMENT`    | `LibraAccount::EPAYEE_CANT_ACCEPT_CURRENCY_TYPE` | The account at `preburn_address` doesn't have a balance resource for `Token`.                         |
///
/// # Related Scripts
/// * `Script::burn_txn_fees`
/// * `Script::burn`
/// * `Script::preburn`

fun cancel_burn<Token>(account: &signer, preburn_address: address) {
    LibraAccount::cancel_burn<Token>(account, preburn_address)
}
}
