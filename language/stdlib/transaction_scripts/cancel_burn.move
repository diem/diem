use 0x0::LibraAccount;

// Cancel the oldest burn request from `preburn_address` and return the funds.
// Fails if the sender does not have a published MintCapability<Token>.
fun main<Token>(preburn_address: address) {
    LibraAccount::cancel_burn<Token>(preburn_address)
}
