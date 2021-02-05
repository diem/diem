script {
use 0x1::Diem;

/// # Summary
/// Script to allow or disallow minting of new coins in a specified currency.  This transaction can
/// only be sent by the Treasury Compliance account.  Turning minting off for a currency will have
/// no effect on coins already in circulation, and coins may still be removed from the system.
///
/// # Technical Description
/// This transaction sets the `can_mint` field of the `Diem::CurrencyInfo<Currency>` resource
/// published under `0xA550C18` to the value of `allow_minting`. Minting of coins if allowed if
/// this field is set to `true` and minting of new coins in `Currency` is disallowed otherwise.
/// This transaction needs to be sent by the Treasury Compliance account.
///
/// # Parameters
/// | Name            | Type      | Description                                                                                                                          |
/// | ------          | ------    | -------------                                                                                                                        |
/// | `Currency`      | Type      | The Move type for the `Currency` whose minting ability is being updated. `Currency` must be an already-registered currency on-chain. |
/// | `account`       | `&signer` | Signer reference of the sending account. Must be the Diem Root account.                                                             |
/// | `allow_minting` | `bool`    | Whether to allow minting of new coins in `Currency`.                                                                                 |
///
/// # Common Abort Conditions
/// | Error Category             | Error Reason                          | Description                                          |
/// | ----------------           | --------------                        | -------------                                        |
/// | `Errors::REQUIRES_ADDRESS` | `CoreAddresses::ETREASURY_COMPLIANCE` | `tc_account` is not the Treasury Compliance account. |
/// | `Errors::NOT_PUBLISHED`    | `Diem::ECURRENCY_INFO`               | `Currency` is not a registered currency on-chain.    |
///
/// # Related Scripts
/// * `Script::update_dual_attestation_limit`
/// * `Script::update_exchange_rate`

fun update_minting_ability<Currency>(
    tc_account: &signer,
    allow_minting: bool
) {
    Diem::update_minting_ability<Currency>(tc_account, allow_minting);
}
}
