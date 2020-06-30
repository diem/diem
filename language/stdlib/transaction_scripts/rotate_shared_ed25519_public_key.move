script {
use 0x1::SharedEd25519PublicKey;

/// (1) Rotate the public key stored in `account`'s `SharedEd25519PublicKey` resource to
/// `new_public_key`
/// (2) Rotate the authentication key using the capability stored in `account`'s
/// `SharedEd25519PublicKey` to a new value derived from `new_public_key`
/// Aborts if `account` does not have a `SharedEd25519PublicKey` resource.
/// Aborts if the length of `new_public_key` is not 32.
fun rotate_shared_ed25519_public_key(account: &signer, public_key: vector<u8>) {
    SharedEd25519PublicKey::rotate_key(account, public_key)
}
}
