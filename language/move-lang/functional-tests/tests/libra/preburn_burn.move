// Test the end-to-end preburn-burn flow

//! account: preburner, 0Coin1

// create 100 Coin1's for the preburner. we can't do this using the //! account macro because it
// doesn't increment the market cap appropriately
//! sender: blessed
script {
use 0x0::Coin1::Coin1;
use 0x0::Libra;
use 0x0::LibraAccount;
fun main(account: &signer) {
    let coin = Libra::mint<Coin1>(account, 100);
    LibraAccount::deposit(account, {{preburner}}, coin);
}
}

// check: MintEvent
// check: EXECUTED

// register the sender as a preburn entity
//! new-transaction
//! sender: preburner
//! gas-currency: Coin1
script {
use 0x0::Coin1::Coin1;
use 0x0::Libra;
fun main(account: &signer) {
    Libra::publish_preburn(account, Libra::new_preburn<Coin1>())
}
}

// check: EXECUTED

// perform a preburn
//! new-transaction
//! sender: preburner
//! gas-currency: Coin1
script {
use 0x0::Coin1::Coin1;
use 0x0::Libra;
use 0x0::LibraAccount;
use 0x0::Transaction;
fun main(account: &signer) {
    let with_cap = LibraAccount::extract_withdraw_capability(account);
    let coin = LibraAccount::withdraw_from<Coin1>(&with_cap, 100);
    LibraAccount::restore_withdraw_capability(with_cap);
    let old_market_cap = Libra::market_cap<Coin1>();
    // send the coins to the preburn bucket. market cap should not be affected, but the preburn
    // bucket should increase in size by 100
    Libra::preburn_to<Coin1>(account, coin);
    Transaction::assert(Libra::market_cap<Coin1>() == old_market_cap, 8002);
    Transaction::assert(Libra::preburn_value<Coin1>() == 100, 8003);
}
}

// check: PreburnEvent
// check: EXECUTED

// perform the burn from the Association account
//! new-transaction
//! sender: blessed
script {
use 0x0::Coin1::Coin1;
use 0x0::Libra;
use 0x0::Transaction;
fun main(account: &signer) {
    let old_market_cap = Libra::market_cap<Coin1>();
    // do the burn. the market cap should now decrease, and the preburn bucket should be empty
    Libra::burn<Coin1>(account, {{preburner}});
    Transaction::assert(Libra::market_cap<Coin1>() == old_market_cap - 100, 8004);
    Transaction::assert(Libra::preburn_value<Coin1>() == 0, 8005);
}
}

// check: BurnEvent
// check: EXECUTED
