script {
use 0x1::SlidingNonce;
use 0x1::DualAttestation;

/// # Summary
/// Update the dual attestation limit on-chain. Defined in terms of micro-LBR.  The transaction can
/// only be sent by the Treasury Compliance account.  After this transaction all inter-VASP
/// payments over this limit must be checked for dual attestation.
///
/// # Technical Description
/// Updates the `micro_lbr_limit` field of the `DualAttestation::Limit` resource published under
/// `0xA550C18`. The amount is set in micro-LBR.
///
/// # Parameters
/// | Name                  | Type      | Description                                                                                               |
/// | ------                | ------    | -------------                                                                                             |
/// | `tc_account`          | `&signer` | The signer reference of the sending account of this transaction. Must be the Treasury Compliance account. |
/// | `sliding_nonce`       | `u64`     | The `sliding_nonce` (see: `SlidingNonce`) to be used for this transaction.                                |
/// | `new_micro_lbr_limit` | `u64`     | The new dual attestation limit to be used on-chain.                                                       |
///
/// # Common Abort Conditions
/// | Error Category             | Error Reason                            | Description                                                                                |
/// | ----------------           | --------------                          | -------------                                                                              |
/// | `Errors::NOT_PUBLISHED`    | `SlidingNonce::ESLIDING_NONCE`          | A `SlidingNonce` resource is not published under `tc_account`.                             |
/// | `Errors::INVALID_ARGUMENT` | `SlidingNonce::ENONCE_TOO_OLD`          | The `sliding_nonce` is too old and it's impossible to determine if it's duplicated or not. |
/// | `Errors::INVALID_ARGUMENT` | `SlidingNonce::ENONCE_TOO_NEW`          | The `sliding_nonce` is too far in the future.                                              |
/// | `Errors::INVALID_ARGUMENT` | `SlidingNonce::ENONCE_ALREADY_RECORDED` | The `sliding_nonce` has been previously recorded.                                          |
/// | `Errors::REQUIRES_ADDRESS` | `CoreAddresses::ETREASURY_COMPLIANCE`   | `tc_account` is not the Treasury Compliance account.                                       |
///
/// # Related Scripts
/// * `Scripts::update_exchange_rate`
/// * `Scripts::update_minting_ability`

fun update_dual_attestation_limit(
    tc_account: &signer,
    sliding_nonce: u64,
    new_micro_lbr_limit: u64
) {
    SlidingNonce::record_nonce_or_abort(tc_account, sliding_nonce);
    DualAttestation::set_microlibra_limit(tc_account, new_micro_lbr_limit);
}
}
