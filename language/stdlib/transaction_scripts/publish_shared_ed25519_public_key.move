script {
use 0x1::SharedEd25519PublicKey;

/// (1) Rotate the authentication key of the sender to `public_key`
/// (2) Publish a resource containing a 32-byte ed25519 public key and the rotation capability
///     of the sender under the sender's address.
/// Aborts if the sender already has a `SharedEd25519PublicKey` resource.
/// Aborts if the length of `new_public_key` is not 32.
fun publish_shared_ed25519_public_key(account: &signer, public_key: vector<u8>) {
    SharedEd25519PublicKey::publish(account, public_key)
}
}
