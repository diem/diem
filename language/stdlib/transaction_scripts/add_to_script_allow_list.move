script {
use 0x1::DiemTransactionPublishingOption;
use 0x1::SlidingNonce;

/// # Summary
/// Adds a script hash to the transaction allowlist. This transaction
/// can only be sent by the Diem Root account. Scripts with this hash can be
/// sent afterward the successful execution of this script.
///
/// # Technical Description
///
/// The sending account (`dr_account`) must be the Diem Root account. The script allow
/// list must not already hold the script `hash` being added. The `sliding_nonce` must be
/// a valid nonce for the Diem Root account. After this transaction has executed
/// successfully a reconfiguration will be initiated, and the on-chain config
/// `DiemTransactionPublishingOption::DiemTransactionPublishingOption`'s
/// `script_allow_list` field will contain the new script `hash` and transactions
/// with this `hash` can be successfully sent to the network.
///
/// # Parameters
/// | Name            | Type         | Description                                                                                     |
/// | ------          | ------       | -------------                                                                                   |
/// | `dr_account`    | `&signer`    | The signer reference of the sending account of this transaction. Must be the Diem Root signer. |
/// | `hash`          | `vector<u8>` | The hash of the script to be added to the script allowlist.                                     |
/// | `sliding_nonce` | `u64`        | The `sliding_nonce` (see: `SlidingNonce`) to be used for this transaction.                      |
///
/// # Common Abort Conditions
/// | Error Category             | Error Reason                                                           | Description                                                                                |
/// | ----------------           | --------------                                                         | -------------                                                                              |
/// | `Errors::NOT_PUBLISHED`    | `SlidingNonce::ESLIDING_NONCE`                                         | A `SlidingNonce` resource is not published under `dr_account`.                             |
/// | `Errors::INVALID_ARGUMENT` | `SlidingNonce::ENONCE_TOO_OLD`                                         | The `sliding_nonce` is too old and it's impossible to determine if it's duplicated or not. |
/// | `Errors::INVALID_ARGUMENT` | `SlidingNonce::ENONCE_TOO_NEW`                                         | The `sliding_nonce` is too far in the future.                                              |
/// | `Errors::INVALID_ARGUMENT` | `SlidingNonce::ENONCE_ALREADY_RECORDED`                                | The `sliding_nonce` has been previously recorded.                                          |
/// | `Errors::REQUIRES_ADDRESS` | `CoreAddresses::EDIEM_ROOT`                                           | The sending account is not the Diem Root account.                                         |
/// | `Errors::REQUIRES_ROLE`    | `Roles::EDIEM_ROOT`                                                   | The sending account is not the Diem Root account.                                         |
/// | `Errors::INVALID_ARGUMENT` | `DiemTransactionPublishingOption::EINVALID_SCRIPT_HASH`               | The script `hash` is an invalid length.                                                    |
/// | `Errors::INVALID_ARGUMENT` | `DiemTransactionPublishingOption::EALLOWLIST_ALREADY_CONTAINS_SCRIPT` | The on-chain allowlist already contains the script `hash`.                                 |

fun add_to_script_allow_list(dr_account: &signer, hash: vector<u8>, sliding_nonce: u64,) {
    SlidingNonce::record_nonce_or_abort(dr_account, sliding_nonce);
    DiemTransactionPublishingOption::add_to_script_allow_list(dr_account, hash)
}
}
