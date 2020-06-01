script {
use 0x0::Libra;
use 0x0::SlidingNonce;

/// Permanently destroy the `Token`s stored in the oldest burn request under the `Preburn` resource.
/// This will only succeed if `account` has a `MintCapability<Token>`, a `Preburn<Token>` resource
/// exists under `preburn_address`, and there is a pending burn request.
/// sliding_nonce is a unique nonce for operation, see sliding_nonce.move for details
fun main<Token>(account: &signer, sliding_nonce: u64, preburn_address: address) {
    SlidingNonce::record_nonce_or_abort(account, sliding_nonce);
    Libra::burn<Token>(account, preburn_address)
}
}
