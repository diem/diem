script {
use 0x0::LibraAccount;
use 0x0::SlidingNonce;

/// Script for un-freezing account by authorized initiator
/// sliding_nonce is a unique nonce for operation, see sliding_nonce.move for details
fun main(account: &signer, sliding_nonce: u64, to_unfreeze_account: address) {
    SlidingNonce::record_nonce_or_abort(account, sliding_nonce);
    LibraAccount::unfreeze_account(account,  to_unfreeze_account);
}
}
