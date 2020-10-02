script {
use 0x1::LibraVersion;
use 0x1::SlidingNonce;

///  # Summary
/// Updates the Libra major version that is stored on-chain and is used by the VM.  This
/// transaction can only be sent from the Libra Root account.
///
/// # Technical Description
/// Updates the `LibraVersion` on-chain config and emits a `LibraConfig::NewEpochEvent` to trigger
/// a reconfiguration of the system. The `major` version that is passed in must be strictly greater
/// than the current major version held on-chain. The VM reads this information and can use it to
/// preserve backwards compatibility with previous major versions of the VM.
///
/// # Parameters
/// | Name            | Type      | Description                                                                |
/// | ------          | ------    | -------------                                                              |
/// | `account`       | `&signer` | Signer reference of the sending account. Must be the Libra Root account.   |
/// | `sliding_nonce` | `u64`     | The `sliding_nonce` (see: `SlidingNonce`) to be used for this transaction. |
/// | `major`         | `u64`     | The `major` version of the VM to be used from this transaction on.         |
///
/// # Common Abort Conditions
/// | Error Category             | Error Reason                                  | Description                                                                                |
/// | ----------------           | --------------                                | -------------                                                                              |
/// | `Errors::NOT_PUBLISHED`    | `SlidingNonce::ESLIDING_NONCE`                | A `SlidingNonce` resource is not published under `account`.                                |
/// | `Errors::INVALID_ARGUMENT` | `SlidingNonce::ENONCE_TOO_OLD`                | The `sliding_nonce` is too old and it's impossible to determine if it's duplicated or not. |
/// | `Errors::INVALID_ARGUMENT` | `SlidingNonce::ENONCE_TOO_NEW`                | The `sliding_nonce` is too far in the future.                                              |
/// | `Errors::INVALID_ARGUMENT` | `SlidingNonce::ENONCE_ALREADY_RECORDED`       | The `sliding_nonce` has been previously recorded.                                          |
/// | `Errors::REQUIRES_ADDRESS` | `CoreAddresses::ELIBRA_ROOT`                  | `account` is not the Libra Root account.                                                   |
/// | `Errors::INVALID_ARGUMENT` | `LibraVersion::EINVALID_MAJOR_VERSION_NUMBER` | `major` is less-than or equal to the current major version stored on-chain.                |

fun update_libra_version(account: &signer, sliding_nonce: u64, major: u64) {
    SlidingNonce::record_nonce_or_abort(account, sliding_nonce);
    LibraVersion::set(account, major)
}
}
