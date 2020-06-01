script {
use 0x0::Libra;
use 0x0::SlidingNonce;

// Permanently destroy the `Token`s stored in the oldest burn request under the `Preburn` resource.
// This will only succeed if the sender has a `MintCapability<Token>` stored under their account, a
// `Preburn<Token>` resource exists under `preburn_address`, and there is a pending burn request.
// sliding_nonce is a unique nonce for operation, see sliding_nonce.move for details
fun main<Token>(sliding_nonce: u64, preburn_address: address) {
    SlidingNonce::record_nonce_or_abort(sliding_nonce);
    Libra::burn<Token>(preburn_address)
}
}
