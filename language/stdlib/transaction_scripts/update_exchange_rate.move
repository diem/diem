script {
use 0x1::Libra;
use 0x1::FixedPoint32;
use 0x1::SlidingNonce;

/// # Summary
/// Update the rough on-chain exchange rate between a specified currency and LBR (as a conversion
/// to micro-LBR). The transaction can only be sent by the Treasury Compliance account. After this
/// transaction the updated exchange rate will be used for normalization of gas prices, and for
/// dual attestation checking.
///
/// # Technical Description
/// Updates the on-chain exchange rate from the given `Currency` to micro-LBR.  The exchange rate
/// is given by `new_exchange_rate_numerator/new_exchange_rate_denominator`.
///
/// # Parameters
/// | Name                            | Type      | Description                                                                                                                        |
/// | ------                          | ------    | -------------                                                                                                                      |
/// | `Currency`                      | Type      | The Move type for the `Currency` whose exchange rate is being updated. `Currency` must be an already-registered currency on-chain. |
/// | `tc_account`                    | `&signer` | The signer reference of the sending account of this transaction. Must be the Treasury Compliance account.                          |
/// | `sliding_nonce`                 | `u64`     | The `sliding_nonce` (see: `SlidingNonce`) to be used for the transaction.                                                          |
/// | `new_exchange_rate_numerator`   | `u64`     | The numerator for the new to micro-LBR exchange rate for `Currency`.                                                               |
/// | `new_exchange_rate_denominator` | `u64`     | The denominator for the new to micro-LBR exchange rate for `Currency`.                                                             |
///
/// # Common Abort Conditions
/// | Error Category             | Error Reason                            | Description                                                                                |
/// | ----------------           | --------------                          | -------------                                                                              |
/// | `Errors::INVALID_ARGUMENT` | `SlidingNonce::ENONCE_TOO_OLD`          | The `sliding_nonce` is too old and it's impossible to determine if it's duplicated or not. |
/// | `Errors::INVALID_ARGUMENT` | `SlidingNonce::ENONCE_TOO_NEW`          | The `sliding_nonce` is too far in the future.                                              |
/// | `Errors::INVALID_ARGUMENT` | `SlidingNonce::ENONCE_ALREADY_RECORDED` | The `sliding_nonce` has been previously recorded.                                          |
/// | `Errors::REQUIRES_ADDRESS` | `CoreAddresses::ETREASURY_COMPLIANCE`   | `tc_account` is not the Treasury Compliance account.                                       |
/// | `Errors::INVALID_ARGUMENT` | `FixedPoint32::EDENOMINATOR`            | `new_exchange_rate_denominator` is zero.                                                   |
/// | `Errors::INVALID_ARGUMENT` | `FixedPoint32::ERATIO_OUT_OF_RANGE`     | The quotient is unrepresentable as a `FixedPoint32`.                                       |
/// | `Errors::LIMIT_EXCEEDED`   | `FixedPoint32::ERATIO_OUT_OF_RANGE`     | The quotient is unrepresentable as a `FixedPoint32`.                                       |
///
/// # Related Scripts
/// * `Scripts::update_dual_attestation_limit`
/// * `Scripts::update_minting_ability`

fun update_exchange_rate<Currency>(
    tc_account: &signer,
    sliding_nonce: u64,
    new_exchange_rate_numerator: u64,
    new_exchange_rate_denominator: u64,
) {
    SlidingNonce::record_nonce_or_abort(tc_account, sliding_nonce);
    let rate = FixedPoint32::create_from_rational(
        new_exchange_rate_numerator,
        new_exchange_rate_denominator,
    );
    Libra::update_lbr_exchange_rate<Currency>(tc_account, rate);
}
}
