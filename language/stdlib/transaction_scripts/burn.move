script {
use 0x0::Libra;

// Permanently destroy the `Token`s stored in the oldest burn request under the `Preburn` resource.
// This will only succeed if the sender has a `MintCapability<Token>` stored under their account, a
// `Preburn<Token>` resource exists under `preburn_address`, and there is a pending burn request.
fun main<Token>(preburn_address: address) {
    Libra::burn<Token>(preburn_address)
}
}
