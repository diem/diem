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
/// | `Errors::LIMIT_EXCEEDED`      | `LibraAccount::EDEPOSIT_EXCEEDS_LIMITS`          | The depositing of the funds held in the prebun area would exceed the `account`'s account limits.      |
/// | `Errors::INVALID_STATE`       | `DualAttestation::EPAYEE_COMPLIANCE_KEY_NOT_SET` | The `account` does not have a compliance key set on it but dual attestion checking was performed.     |
///
/// # Related Scripts
/// * `Script::burn_txn_fees`
/// * `Script::burn`
/// * `Script::preburn`

fun cancel_burn<Token>(account: &signer, preburn_address: address) {
    LibraAccount::cancel_burn<Token>(account, preburn_address)
}

spec fun cancel_burn {
    use 0x1::CoreAddresses;
    use 0x1::Errors;
    use 0x1::Libra;

    include LibraAccount::TransactionChecks{sender: account}; // properties checked by the prologue.
    include LibraAccount::CancelBurnAbortsIf<Token>;

    let preburn_value_at_addr = global<Libra::Preburn<Token>>(preburn_address).to_burn.value;
    let total_preburn_value =
        global<Libra::CurrencyInfo<Token>>(CoreAddresses::CURRENCY_INFO_ADDRESS()).preburn_value;
    let balance_at_addr = LibraAccount::balance<Token>(preburn_address);

    /// The value stored at `Libra::Preburn` under `preburn_address` should become zero.
    ensures preburn_value_at_addr == 0;

    /// The total value of preburn for `Token` should decrease by the preburned amount.
    ensures total_preburn_value == old(total_preburn_value) - old(preburn_value_at_addr);

    /// The balance of `Token` at `preburn_address` should increase by the preburned amount.
    ensures balance_at_addr == old(balance_at_addr) + old(preburn_value_at_addr);

    aborts_with [check]
        Errors::REQUIRES_CAPABILITY,
        Errors::NOT_PUBLISHED,
        Errors::INVALID_ARGUMENT,
        Errors::LIMIT_EXCEEDED,
        Errors::INVALID_STATE;

    /// **Access Control:**
    /// Only the account with the burn capability can cancel burning [[H3]][PERMISSION].
    include Libra::AbortsIfNoBurnCapability<Token>{account: account};
}
}
