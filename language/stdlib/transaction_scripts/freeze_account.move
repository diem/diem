script {
use 0x1::AccountFreezing;
use 0x1::SlidingNonce;

/// # Summary
/// Freezes the account at `address`. The sending account of this transaction
/// must be the Treasury Compliance account. The account being frozen cannot be
/// the Diem Root or Treasury Compliance account. After the successful
/// execution of this transaction no transactions may be sent from the frozen
/// account, and the frozen account may not send or receive coins.
///
/// # Technical Description
/// Sets the `AccountFreezing::FreezingBit` to `true` and emits a
/// `AccountFreezing::FreezeAccountEvent`. The transaction sender must be the
/// Treasury Compliance account, but the account at `to_freeze_account` must
/// not be either `0xA550C18` (the Diem Root address), or `0xB1E55ED` (the
/// Treasury Compliance address). Note that this is a per-account property
/// e.g., freezing a Parent VASP will not effect the status any of its child
/// accounts and vice versa.
///
///
/// ## Events
/// Successful execution of this transaction will emit a `AccountFreezing::FreezeAccountEvent` on
/// the `freeze_event_handle` held in the `AccountFreezing::FreezeEventsHolder` resource published
/// under `0xA550C18` with the `frozen_address` being the `to_freeze_account`.
///
/// # Parameters
/// | Name                | Type      | Description                                                                                               |
/// | ------              | ------    | -------------                                                                                             |
/// | `tc_account`        | `&signer` | The signer reference of the sending account of this transaction. Must be the Treasury Compliance account. |
/// | `sliding_nonce`     | `u64`     | The `sliding_nonce` (see: `SlidingNonce`) to be used for this transaction.                                |
/// | `to_freeze_account` | `address` | The account address to be frozen.                                                                         |
///
/// # Common Abort Conditions
/// | Error Category             | Error Reason                                 | Description                                                                                |
/// | ----------------           | --------------                               | -------------                                                                              |
/// | `Errors::NOT_PUBLISHED`    | `SlidingNonce::ESLIDING_NONCE`               | A `SlidingNonce` resource is not published under `tc_account`.                             |
/// | `Errors::INVALID_ARGUMENT` | `SlidingNonce::ENONCE_TOO_OLD`               | The `sliding_nonce` is too old and it's impossible to determine if it's duplicated or not. |
/// | `Errors::INVALID_ARGUMENT` | `SlidingNonce::ENONCE_TOO_NEW`               | The `sliding_nonce` is too far in the future.                                              |
/// | `Errors::INVALID_ARGUMENT` | `SlidingNonce::ENONCE_ALREADY_RECORDED`      | The `sliding_nonce` has been previously recorded.                                          |
/// | `Errors::REQUIRES_ADDRESS` | `CoreAddresses::ETREASURY_COMPLIANCE`        | The sending account is not the Treasury Compliance account.                                |
/// | `Errors::REQUIRES_ROLE`    | `Roles::ETREASURY_COMPLIANCE`                | The sending account is not the Treasury Compliance account.                                |
/// | `Errors::INVALID_ARGUMENT` | `AccountFreezing::ECANNOT_FREEZE_TC`         | `to_freeze_account` was the Treasury Compliance account (`0xB1E55ED`).                     |
/// | `Errors::INVALID_ARGUMENT` | `AccountFreezing::ECANNOT_FREEZE_DIEM_ROOT` | `to_freeze_account` was the Diem Root account (`0xA550C18`).                              |
///
/// # Related Scripts
/// * `Script::unfreeze_account`

fun freeze_account(tc_account: &signer, sliding_nonce: u64, to_freeze_account: address) {
    SlidingNonce::record_nonce_or_abort(tc_account, sliding_nonce);
    AccountFreezing::freeze_account(tc_account, to_freeze_account);
}
}
