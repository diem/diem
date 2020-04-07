use 0x0::Libra;
use 0x0::LibraAccount;

// Preburn `amount` `Token`s from the sender's account.
// This will only succeed if the sender already has a published `Preburn<Token>` resource.
fun main<Token>(amount: u64) {
    Libra::preburn_to_sender<Token>(LibraAccount::withdraw_from_sender(amount))
}
