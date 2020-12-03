script {
use 0x1::Diem;
use 0x1::SlidingNonce;

/// # Summary
/// Burns all coins held in the preburn resource at the specified
/// preburn address and removes them from the system. The sending account must
/// be the Treasury Compliance account.
/// The account that holds the preburn resource will normally be a Designated
/// Dealer, but there are no enforced requirements that it be one.
///
/// # Technical Description
/// This transaction permanently destroys all the coins of `Token` type
/// stored in the `Diem::Preburn<Token>` resource published under the
/// `preburn_address` account address.
///
/// This transaction will only succeed if the sending `account` has a
/// `Diem::BurnCapability<Token>`, and a `Diem::Preburn<Token>` resource
/// exists under `preburn_address`, with a non-zero `to_burn` field. After the successful execution
/// of this transaction the `total_value` field in the
/// `Diem::CurrencyInfo<Token>` resource published under `0xA550C18` will be
/// decremented by the value of the `to_burn` field of the preburn resource
/// under `preburn_address` immediately before this transaction, and the
/// `to_burn` field of the preburn resource will have a zero value.
///
/// ## Events
/// The successful execution of this transaction will emit a `Diem::BurnEvent` on the event handle
/// held in the `Diem::CurrencyInfo<Token>` resource's `burn_events` published under
/// `0xA550C18`.
///
/// # Parameters
/// | Name              | Type      | Description                                                                                                                  |
/// | ------            | ------    | -------------                                                                                                                |
/// | `Token`           | Type      | The Move type for the `Token` currency being burned. `Token` must be an already-registered currency on-chain.                |
/// | `tc_account`      | `&signer` | The signer reference of the sending account of this transaction, must have a burn capability for `Token` published under it. |
/// | `sliding_nonce`   | `u64`     | The `sliding_nonce` (see: `SlidingNonce`) to be used for this transaction.                                                   |
/// | `preburn_address` | `address` | The address where the coins to-be-burned are currently held.                                                                 |
///
/// # Common Abort Conditions
/// | Error Category                | Error Reason                            | Description                                                                                           |
/// | ----------------              | --------------                          | -------------                                                                                         |
/// | `Errors::NOT_PUBLISHED`       | `SlidingNonce::ESLIDING_NONCE`          | A `SlidingNonce` resource is not published under `account`.                                           |
/// | `Errors::INVALID_ARGUMENT`    | `SlidingNonce::ENONCE_TOO_OLD`          | The `sliding_nonce` is too old and it's impossible to determine if it's duplicated or not.            |
/// | `Errors::INVALID_ARGUMENT`    | `SlidingNonce::ENONCE_TOO_NEW`          | The `sliding_nonce` is too far in the future.                                                         |
/// | `Errors::INVALID_ARGUMENT`    | `SlidingNonce::ENONCE_ALREADY_RECORDED` | The `sliding_nonce` has been previously recorded.                                                     |
/// | `Errors::REQUIRES_CAPABILITY` | `Diem::EBURN_CAPABILITY`               | The sending `account` does not have a `Diem::BurnCapability<Token>` published under it.              |
/// | `Errors::NOT_PUBLISHED`       | `Diem::EPREBURN`                       | The account at `preburn_address` does not have a `Diem::Preburn<Token>` resource published under it. |
/// | `Errors::INVALID_STATE`       | `Diem::EPREBURN_EMPTY`                 | The `Diem::Preburn<Token>` resource is empty (has a value of 0).                                     |
/// | `Errors::NOT_PUBLISHED`       | `Diem::ECURRENCY_INFO`                 | The specified `Token` is not a registered currency on-chain.                                          |
///
/// # Related Scripts
/// * `Script::burn_txn_fees`
/// * `Script::cancel_burn`
/// * `Script::preburn`

fun burn<Token>(account: &signer, sliding_nonce: u64, preburn_address: address) {
    SlidingNonce::record_nonce_or_abort(account, sliding_nonce);
    Diem::burn<Token>(account, preburn_address)
}
spec fun burn {
    use 0x1::Errors;
    use 0x1::DiemAccount;

    include DiemAccount::TransactionChecks{sender: account}; // properties checked by the prologue.
    include SlidingNonce::RecordNonceAbortsIf{ seq_nonce: sliding_nonce };
    include Diem::BurnAbortsIf<Token>;
    include Diem::BurnEnsures<Token>;

    aborts_with [check]
        Errors::INVALID_ARGUMENT,
        Errors::REQUIRES_CAPABILITY,
        Errors::NOT_PUBLISHED,
        Errors::INVALID_STATE,
        Errors::LIMIT_EXCEEDED;

    /// **Access Control:**
    /// Only the account with the burn capability can burn coins [[H3]][PERMISSION].
    include Diem::AbortsIfNoBurnCapability<Token>{account: account};
}
}
