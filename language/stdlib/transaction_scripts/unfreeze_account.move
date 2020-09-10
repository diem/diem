script {
use 0x1::AccountFreezing;
use 0x1::SlidingNonce;

/// Unfreeze account `address`. Initiator must be authorized.
/// `sliding_nonce` is a unique nonce for operation, see sliding_nonce.move for details.
fun unfreeze_account(account: &signer, sliding_nonce: u64, to_unfreeze_account: address) {
    SlidingNonce::record_nonce_or_abort(account, sliding_nonce);
    AccountFreezing::unfreeze_account(account, to_unfreeze_account);
}
}
