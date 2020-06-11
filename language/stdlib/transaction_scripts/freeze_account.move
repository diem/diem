script {
use 0x1::LibraAccount;
use 0x1::SlidingNonce;

/// Script for freezing account by authorized initiator
/// sliding_nonce is a unique nonce for operation, see sliding_nonce.move for details
fun main(account: &signer, sliding_nonce: u64, to_freeze_account: address) {
    SlidingNonce::record_nonce_or_abort(account, sliding_nonce);
    LibraAccount::freeze_account(account, to_freeze_account);
}
}
