// Test the end-to-end preburn-burn flow with the simplest possible scenario: burner and preburner
// are the same entity.

// register the sender as a preburn entity
//! sender: association
use 0x0::LibraCoin;
fun main() {
    let preburn = LibraCoin::new_preburn();
    LibraCoin::publish_preburn(preburn)
}

// check: EXECUTED

// perform a preburn
//! new-transaction
//! sender: association
use 0x0::LibraAccount;
use 0x0::LibraCoin;
use 0x0::Transaction;
fun main() {
    let coin = LibraAccount::withdraw_from_sender(100);
    let old_market_cap = LibraCoin::market_cap();
    // send the coins to the preburn bucket. market cap should not be affected, but the preburn
    // bucket should increase in size by 100
    LibraCoin::preburn(coin);
    Transaction::assert(LibraCoin::market_cap() == old_market_cap, 8002);
    Transaction::assert(LibraCoin::preburn_value() == 100, 8003);
}

// check: EXECUTED

// perform the burn
//! new-transaction
//! sender: association
use 0x0::LibraCoin;
use 0x0::Transaction;
fun main() {
    let old_market_cap = LibraCoin::market_cap();
    // do the burn. the market cap should now decrease, and the preburn bucket should be empty
    LibraCoin::burn(Transaction::sender());
    Transaction::assert(LibraCoin::market_cap() == old_market_cap - 100, 8004);
    Transaction::assert(LibraCoin::preburn_value() == 0, 8005);
}

// check: EXECUTED
