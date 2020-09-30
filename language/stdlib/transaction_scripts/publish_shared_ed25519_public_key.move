script {
use 0x1::SharedEd25519PublicKey;

/// # Summary
/// Rotates the authentication key of the sending account to the
/// newly-specified public key and publishes a new shared authentication key
/// under the sender's account. Any account can send this transaction.
///
/// # Technical Description
/// Rotates the authentication key of the sending account to `public_key`,
/// and publishes a `SharedEd25519PublicKey::SharedEd25519PublicKey` resource
/// containing the 32-byte ed25519 `public_key` and the `LibraAccount::KeyRotationCapability` for
/// `account` under `account`.
///
/// # Parameters
/// | Name         | Type         | Description                                                                               |
/// | ------       | ------       | -------------                                                                             |
/// | `account`    | `&signer`    | The signer reference of the sending account of the transaction.                           |
/// | `public_key` | `vector<u8>` | 32-byte Ed25519 public key for `account`' authentication key to be rotated to and stored. |
///
/// # Common Abort Conditions
/// | Error Category              | Error Reason                                               | Description                                                                                         |
/// | ----------------            | --------------                                             | -------------                                                                                       |
/// | `Errors::INVALID_STATE`     | `LibraAccount::EKEY_ROTATION_CAPABILITY_ALREADY_EXTRACTED` | `account` has already delegated/extracted its `LibraAccount::KeyRotationCapability` resource.       |
/// | `Errors::ALREADY_PUBLISHED` | `SharedEd25519PublicKey::ESHARED_KEY`                      | The `SharedEd25519PublicKey::SharedEd25519PublicKey` resource is already published under `account`. |
/// | `Errors::INVALID_ARGUMENT`  | `SharedEd25519PublicKey::EMALFORMED_PUBLIC_KEY`            | `public_key` is an invalid ed25519 public key.                                                      |
///
/// # Related Scripts
/// * `Script::rotate_shared_ed25519_public_key`

fun publish_shared_ed25519_public_key(account: &signer, public_key: vector<u8>) {
    SharedEd25519PublicKey::publish(account, public_key)
}
spec fun publish_shared_ed25519_public_key {
    use 0x1::Errors;
    use 0x1::LibraAccount;
    use 0x1::Signer;

    include SharedEd25519PublicKey::PublishAbortsIf{key: public_key};
    include SharedEd25519PublicKey::PublishEnsures{key: public_key};

    // TODO: The following line is added to help Prover verifying the
    // "aborts_with [check]" spec. This fact can be inferred from the successful
    // termination of the prologue functions, but Prover cannot figure it out currently.
    requires LibraAccount::exists_at(Signer::spec_address_of(account));

    aborts_with [check]
        Errors::INVALID_STATE,
        Errors::ALREADY_PUBLISHED,
        Errors::INVALID_ARGUMENT;
}
}
