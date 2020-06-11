script {
use 0x1::LibraAccount;
use 0x1::SlidingNonce;
// sliding_nonce is a unique nonce for operation, see sliding_nonce.move for details
fun main(account: &signer, sliding_nonce: u64, new_key: vector<u8>) {
  SlidingNonce::record_nonce_or_abort(account, sliding_nonce);
  let key_rotation_capability = LibraAccount::extract_key_rotation_capability(account);
  LibraAccount::rotate_authentication_key(&key_rotation_capability, new_key);
  LibraAccount::restore_key_rotation_capability(key_rotation_capability);
}
}
