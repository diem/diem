script {
use 0x1::AccountFreezing;
use 0x1::SlidingNonce;

/// # Summary
/// Unfreezes the account at `address`. The sending account of this transaction must be the
/// Treasury Compliance account. After the successful execution of this transaction transactions
/// may be sent from the previously frozen account, and coins may be sent and received.
///
/// # Technical Description
/// Sets the `AccountFreezing::FreezingBit` to `false` and emits a
/// `AccountFreezing::UnFreezeAccountEvent`. The transaction sender must be the Treasury Compliance
/// account. Note that this is a per-account property so unfreezing a Parent VASP will not effect
/// the status any of its child accounts and vice versa.
///
/// ## Events
/// Successful execution of this script will emit a `AccountFreezing::UnFreezeAccountEvent` with
/// the `unfrozen_address` set the `to_unfreeze_account`'s address.
///
/// # Parameters
/// | Name                  | Type      | Description                                                                                               |
/// | ------                | ------    | -------------                                                                                             |
/// | `tc_account`          | `&signer` | The signer reference of the sending account of this transaction. Must be the Treasury Compliance account. |
/// | `sliding_nonce`       | `u64`     | The `sliding_nonce` (see: `SlidingNonce`) to be used for this transaction.                                |
/// | `to_unfreeze_account` | `address` | The account address to be frozen.                                                                         |
///
/// # Common Abort Conditions
/// | Error Category             | Error Reason                            | Description                                                                                |
/// | ----------------           | --------------                          | -------------                                                                              |
/// | `Errors::NOT_PUBLISHED`    | `SlidingNonce::ESLIDING_NONCE`          | A `SlidingNonce` resource is not published under `account`.                                |
/// | `Errors::INVALID_ARGUMENT` | `SlidingNonce::ENONCE_TOO_OLD`          | The `sliding_nonce` is too old and it's impossible to determine if it's duplicated or not. |
/// | `Errors::INVALID_ARGUMENT` | `SlidingNonce::ENONCE_TOO_NEW`          | The `sliding_nonce` is too far in the future.                                              |
/// | `Errors::INVALID_ARGUMENT` | `SlidingNonce::ENONCE_ALREADY_RECORDED` | The `sliding_nonce` has been previously recorded.                                          |
/// | `Errors::REQUIRES_ADDRESS` | `CoreAddresses::ETREASURY_COMPLIANCE`   | The sending account is not the Treasury Compliance account.                                |
///
/// # Related Scripts
/// * `Scripts::freeze_account`

fun unfreeze_account(account: &signer, sliding_nonce: u64, to_unfreeze_account: address) {
    SlidingNonce::record_nonce_or_abort(account, sliding_nonce);
    AccountFreezing::unfreeze_account(account, to_unfreeze_account);
}
}
