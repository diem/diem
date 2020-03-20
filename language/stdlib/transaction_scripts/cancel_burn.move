use 0x0::LBR;
use 0x0::LibraAccount;

// Cancel the oldest burn request from `preburn_address` and return the funds.
// Fails if the sender does not have a published MintCapability.
fun main(preburn_address: address) {
    LibraAccount::cancel_burn<LBR::T>(preburn_address)
}
