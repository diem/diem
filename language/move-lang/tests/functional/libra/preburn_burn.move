// Test the end-to-end preburn-burn flow with the simplest possible scenario: burner and preburner
// are the same entity.

// register the sender as a preburn entity
//! sender: association
use 0x0::LBR;
use 0x0::Libra;
fun main() {
    Libra::publish_preburn(Libra::new_preburn<LBR::T>())
}

// check: EXECUTED

// perform a preburn
//! new-transaction
//! sender: association
use 0x0::LibraAccount;
use 0x0::LBR;
use 0x0::Libra;
use 0x0::Transaction;
fun main() {
    let coin = LibraAccount::withdraw_from_sender<LBR::T>(100);
    let old_market_cap = Libra::market_cap<LBR::T>();
    // send the coins to the preburn bucket. market cap should not be affected, but the preburn
    // bucket should increase in size by 100
    Libra::preburn_to_sender<LBR::T>(coin);
    Transaction::assert(Libra::market_cap<LBR::T>() == old_market_cap, 8002);
    Transaction::assert(Libra::preburn_value<LBR::T>() == 100, 8003);
}

// check: EXECUTED

// perform the burn
//! new-transaction
//! sender: association
use 0x0::LBR;
use 0x0::Libra;
use 0x0::Transaction;
fun main() {
    let old_market_cap = Libra::market_cap<LBR::T>();
    // do the burn. the market cap should now decrease, and the preburn bucket should be empty
    Libra::burn<LBR::T>(Transaction::sender());
    Transaction::assert(Libra::market_cap<LBR::T>() == old_market_cap - 100, 8004);
    Transaction::assert(Libra::preburn_value<LBR::T>() == 0, 8005);
}

// check: EXECUTED
