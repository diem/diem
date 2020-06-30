script {
use 0x1::LibraAccount;

/// Rotate the sender's authentication key to `new_key`.
/// `new_key` should be a 256 bit sha3 hash of an ed25519 public key.
fun rotate_authentication_key(account: &signer, new_key: vector<u8>) {
  let key_rotation_capability = LibraAccount::extract_key_rotation_capability(account);
  LibraAccount::rotate_authentication_key(&key_rotation_capability, new_key);
  LibraAccount::restore_key_rotation_capability(key_rotation_capability);
}
}
