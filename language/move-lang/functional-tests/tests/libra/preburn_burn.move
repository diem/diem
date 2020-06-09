// Test the end-to-end preburn-burn flow

// register the sender as a preburn entity + perform a preburn
//! new-transaction
//! sender: association
script {
use 0x0::Coin1::Coin1;
use 0x0::Libra;
use 0x0::Transaction;
fun main(account: &signer) {
    Libra::publish_preburn_to_account<Coin1>(account, account);
    let coin = Libra::mint<Coin1>(account, 100);
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
    Libra::burn<Coin1>(account, {{association}});
    Transaction::assert(Libra::market_cap<Coin1>() == old_market_cap - 100, 8004);
    Transaction::assert(Libra::preburn_value<Coin1>() == 0, 8005);
}
}

// check: BurnEvent
// check: EXECUTED
