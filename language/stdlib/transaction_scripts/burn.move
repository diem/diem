use 0x0::LBR;
use 0x0::Libra;

// Permanently destroy the coins stored in the oldest burn request under the `Preburn` resource.
// This will only succeed if the sender has a `MintCapability` stored under their account, a
// `Preburn` resource exists under `preburn_address`, and there is a pending burn request.
fun main(preburn_address: address) {
    Libra::burn<LBR::T>(preburn_address)
}
