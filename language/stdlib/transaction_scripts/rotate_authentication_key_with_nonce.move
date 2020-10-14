script {
use 0x1::LibraAccount;
use 0x1::SlidingNonce;

/// # Summary
/// Rotates the sender's authentication key to the supplied new authentication key. May be sent by
/// any account that has a sliding nonce resource published under it (usually this is Treasury
/// Compliance or Libra Root accounts).
///
/// # Technical Description
/// Rotates the `account`'s `LibraAccount::LibraAccount` `authentication_key` field to `new_key`.
/// `new_key` must be a valid ed25519 public key, and `account` must not have previously delegated
/// its `LibraAccount::KeyRotationCapability`.
///
/// # Parameters
/// | Name            | Type         | Description                                                                |
/// | ------          | ------       | -------------                                                              |
/// | `account`       | `&signer`    | Signer reference of the sending account of the transaction.                |
/// | `sliding_nonce` | `u64`        | The `sliding_nonce` (see: `SlidingNonce`) to be used for this transaction. |
/// | `new_key`       | `vector<u8>` | New ed25519 public key to be used for `account`.                           |
///
/// # Common Abort Conditions
/// | Error Category             | Error Reason                                               | Description                                                                                |
/// | ----------------           | --------------                                             | -------------                                                                              |
/// | `Errors::NOT_PUBLISHED`    | `SlidingNonce::ESLIDING_NONCE`                             | A `SlidingNonce` resource is not published under `account`.                                |
/// | `Errors::INVALID_ARGUMENT` | `SlidingNonce::ENONCE_TOO_OLD`                             | The `sliding_nonce` is too old and it's impossible to determine if it's duplicated or not. |
/// | `Errors::INVALID_ARGUMENT` | `SlidingNonce::ENONCE_TOO_NEW`                             | The `sliding_nonce` is too far in the future.                                              |
/// | `Errors::INVALID_ARGUMENT` | `SlidingNonce::ENONCE_ALREADY_RECORDED`                    | The `sliding_nonce` has been previously recorded.                                          |
/// | `Errors::INVALID_STATE`    | `LibraAccount::EKEY_ROTATION_CAPABILITY_ALREADY_EXTRACTED` | `account` has already delegated/extracted its `LibraAccount::KeyRotationCapability`.       |
/// | `Errors::INVALID_ARGUMENT` | `LibraAccount::EMALFORMED_AUTHENTICATION_KEY`              | `new_key` was an invalid length.                                                           |
///
/// # Related Scripts
/// * `Script::rotate_authentication_key`
/// * `Script::rotate_authentication_key_with_nonce_admin`
/// * `Script::rotate_authentication_key_with_recovery_address`

fun rotate_authentication_key_with_nonce(account: &signer, sliding_nonce: u64, new_key: vector<u8>) {
    SlidingNonce::record_nonce_or_abort(account, sliding_nonce);
    let key_rotation_capability = LibraAccount::extract_key_rotation_capability(account);
    LibraAccount::rotate_authentication_key(&key_rotation_capability, new_key);
    LibraAccount::restore_key_rotation_capability(key_rotation_capability);
}
spec fun rotate_authentication_key_with_nonce {
    use 0x1::Signer;
    use 0x1::Errors;

    include LibraAccount::TransactionChecks{sender: account}; // properties checked by the prologue.
    let account_addr = Signer::spec_address_of(account);
    include SlidingNonce::RecordNonceAbortsIf{ seq_nonce: sliding_nonce };
    include LibraAccount::ExtractKeyRotationCapabilityAbortsIf;
    let key_rotation_capability = LibraAccount::spec_get_key_rotation_cap(account_addr);
    include LibraAccount::RotateAuthenticationKeyAbortsIf{cap: key_rotation_capability, new_authentication_key: new_key};

    /// This rotates the authentication key of `account` to `new_key`
    include LibraAccount::RotateAuthenticationKeyEnsures{addr: account_addr, new_authentication_key: new_key};

    aborts_with [check]
        Errors::INVALID_ARGUMENT,
        Errors::INVALID_STATE,
        Errors::NOT_PUBLISHED;

    /// **Access Control:**
    /// The account can rotate its own authentication key unless
    /// it has delegrated the capability [[H17]][PERMISSION][[J17]][PERMISSION].
    include LibraAccount::AbortsIfDelegatedKeyRotationCapability;
}
}
