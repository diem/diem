script {
use 0x1::LibraAccount;

/// # Summary
/// Rotates the transaction sender's authentication key to the supplied new authentication key. May
/// be sent by any account.
///
/// # Technical Description
/// Rotate the `account`'s `LibraAccount::LibraAccount` `authentication_key` field to `new_key`.
/// `new_key` must be a valid ed25519 public key, and `account` must not have previously delegated
/// its `LibraAccount::KeyRotationCapability`.
///
/// # Parameters
/// | Name      | Type         | Description                                                 |
/// | ------    | ------       | -------------                                               |
/// | `account` | `&signer`    | Signer reference of the sending account of the transaction. |
/// | `new_key` | `vector<u8>` | New ed25519 public key to be used for `account`.            |
///
/// # Common Abort Conditions
/// | Error Category             | Error Reason                                               | Description                                                                              |
/// | ----------------           | --------------                                             | -------------                                                                            |
/// | `Errors::INVALID_STATE`    | `LibraAccount::EKEY_ROTATION_CAPABILITY_ALREADY_EXTRACTED` | `account` has already delegated/extracted its `LibraAccount::KeyRotationCapability`.     |
/// | `Errors::INVALID_ARGUMENT` | `LibraAccount::EMALFORMED_AUTHENTICATION_KEY`              | `new_key` was an invalid length.                                                         |
///
/// # Related Scripts
/// * `Script::rotate_authentication_key_with_nonce`
/// * `Script::rotate_authentication_key_with_nonce_admin`
/// * `Script::rotate_authentication_key_with_recovery_address`

fun rotate_authentication_key(account: &signer, new_key: vector<u8>) {
    let key_rotation_capability = LibraAccount::extract_key_rotation_capability(account);
    LibraAccount::rotate_authentication_key(&key_rotation_capability, new_key);
    LibraAccount::restore_key_rotation_capability(key_rotation_capability);
}
spec fun rotate_authentication_key {
    use 0x1::Signer;
    use 0x1::Errors;

    let account_addr = Signer::spec_address_of(account);
    include LibraAccount::ExtractKeyRotationCapabilityAbortsIf;
    let key_rotation_capability = LibraAccount::spec_get_key_rotation_cap(account_addr);
    include LibraAccount::RotateAuthenticationKeyAbortsIf{cap: key_rotation_capability, new_authentication_key: new_key};

    /// This rotates the authentication key of `account` to `new_key`
    include LibraAccount::RotateAuthenticationKeyEnsures{addr: account_addr, new_authentication_key: new_key};

    // TODO: The following line is added to help Prover verifying the
    // "aborts_with [check]" spec. This fact can be inferred from the successful
    // termination of the prologue functions, but Prover cannot figure it out currently.
    requires LibraAccount::exists_at(account_addr);

    aborts_with [check]
        Errors::INVALID_STATE,
        Errors::INVALID_ARGUMENT;
}
}
