script {
use 0x0::LibraAccount;
use 0x0::SlidingNonce;

/// Script for freezing account by authorized initiator
/// sliding_nonce is a unique nonce for operation, see sliding_nonce.move for details
fun main(account: &signer, sliding_nonce: u64, to_freeze_account: address) {
    SlidingNonce::record_nonce_or_abort(account, sliding_nonce);
    LibraAccount::freeze_account(to_freeze_account);
}
}
