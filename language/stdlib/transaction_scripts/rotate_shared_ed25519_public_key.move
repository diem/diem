script {
use 0x0::SharedEd25519PublicKey;

// (1) rotate the public key stored in the sender's `SharedEd25519PublicKey` resource to
// `new_public_key`
// (2) rotate the authentication key using the capability stored in the sender's
// `SharedEd25519PublicKey` to a new value derived from `new_public_key`
// Aborts if the sender does not have a `SharedEd25519PublicKey` resource.
// Aborts if the length of `new_public_key` is not 32.
fun main(public_key: vector<u8>) {
    SharedEd25519PublicKey::rotate_sender_key(public_key)
}
}
