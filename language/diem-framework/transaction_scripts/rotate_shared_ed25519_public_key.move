script {
use 0x1::SharedEd25519PublicKey;

/// # Summary
/// Rotates the authentication key in a `SharedEd25519PublicKey`. This transaction can be sent by
/// any account that has previously published a shared ed25519 public key using
/// `Script::publish_shared_ed25519_public_key`.
///
/// # Technical Description
/// This first rotates the public key stored in `account`'s
/// `SharedEd25519PublicKey::SharedEd25519PublicKey` resource to `public_key`, after which it
/// rotates the authentication key using the capability stored in `account`'s
/// `SharedEd25519PublicKey::SharedEd25519PublicKey` to a new value derived from `public_key`
///
/// # Parameters
/// | Name         | Type         | Description                                                     |
/// | ------       | ------       | -------------                                                   |
/// | `account`    | `&signer`    | The signer reference of the sending account of the transaction. |
/// | `public_key` | `vector<u8>` | 32-byte Ed25519 public key.                                     |
///
/// # Common Abort Conditions
/// | Error Category             | Error Reason                                    | Description                                                                                   |
/// | ----------------           | --------------                                  | -------------                                                                                 |
/// | `Errors::NOT_PUBLISHED`    | `SharedEd25519PublicKey::ESHARED_KEY`           | A `SharedEd25519PublicKey::SharedEd25519PublicKey` resource is not published under `account`. |
/// | `Errors::INVALID_ARGUMENT` | `SharedEd25519PublicKey::EMALFORMED_PUBLIC_KEY` | `public_key` is an invalid ed25519 public key.                                                |
///
/// # Related Scripts
/// * `Script::publish_shared_ed25519_public_key`

fun rotate_shared_ed25519_public_key(account: &signer, public_key: vector<u8>) {
    SharedEd25519PublicKey::rotate_key(account, public_key)
}
spec fun rotate_shared_ed25519_public_key {
    use 0x1::Errors;
    use 0x1::DiemAccount;

    include DiemAccount::TransactionChecks{sender: account}; // properties checked by the prologue.
    include SharedEd25519PublicKey::RotateKeyAbortsIf{new_public_key: public_key};
    include SharedEd25519PublicKey::RotateKeyEnsures{new_public_key: public_key};

    aborts_with [check]
        Errors::NOT_PUBLISHED,
        Errors::INVALID_ARGUMENT;
}
}
