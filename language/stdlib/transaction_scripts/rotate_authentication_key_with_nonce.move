script {
use 0x1::LibraAccount;
use 0x1::SlidingNonce;

/// Rotate the sender's authentication key to `new_key`.
/// `new_key` should be a 256 bit sha3 hash of an ed25519 public key. This script also takes
/// `sliding_nonce`, as a unique nonce for this operation. See sliding_nonce.move for details.
fun rotate_authentication_key_with_nonce(account: &signer, sliding_nonce: u64, new_key: vector<u8>) {
  SlidingNonce::record_nonce_or_abort(account, sliding_nonce);
  let key_rotation_capability = LibraAccount::extract_key_rotation_capability(account);
  LibraAccount::rotate_authentication_key(&key_rotation_capability, new_key);
  LibraAccount::restore_key_rotation_capability(key_rotation_capability);
}
}
