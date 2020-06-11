script {
use 0x1::LibraAccount;

// Cancel the oldest burn request from `preburn_address` and return the funds.
// Fails if the sender does not have a published MintCapability<Token>.
fun main<Token>(account: &signer, preburn_address: address) {
    LibraAccount::cancel_burn<Token>(account, preburn_address)
}
}
