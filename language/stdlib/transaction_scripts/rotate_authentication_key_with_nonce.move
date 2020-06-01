script {
use 0x0::LibraAccount;
use 0x0::SlidingNonce;
// sliding_nonce is a unique nonce for operation, see sliding_nonce.move for details
fun main(sliding_nonce: u64, new_key: vector<u8>) {
  SlidingNonce::record_nonce_or_abort(sliding_nonce);
  LibraAccount::rotate_authentication_key(new_key)
}
}
