use 0x0::LBR;
use 0x0::Libra;
use 0x0::LibraAccount;

// Preburn `amount` coins from the sender's account.
// This will only succeed if the sender already has a published `Preburn` resource.
fun main(amount: u64) {
    Libra::preburn_to_sender<LBR::T>(LibraAccount::withdraw_from_sender<LBR::T>(amount))
}
